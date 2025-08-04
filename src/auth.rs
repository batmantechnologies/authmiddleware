use std::future::{ready, Ready};
use log;
use std::rc::Rc;
use std::cell::RefCell;


use actix_web::body::EitherBody;
use actix_web::dev::{self, ServiceRequest, ServiceResponse};
use actix_web::dev::{Service, Transform};
use actix_web::{Error, HttpMessage};
use actix_web::http::header::{COOKIE};
use futures_util::future::LocalBoxFuture;
use reqwest::{self, header};

pub use crate::utils::{AuthData, AuthInfo, HttpClient};

pub struct AuthenticateMiddlewareFactory {
    auth_data: Rc<AuthData>,
}

impl AuthenticateMiddlewareFactory {
    pub fn new(auth_data: AuthData) -> Self {
        AuthenticateMiddlewareFactory {
            auth_data: Rc::new(auth_data),
        }
    }
}

impl<S: 'static, B> Transform<S, ServiceRequest> for AuthenticateMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckLoginMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CheckLoginMiddleware {
            auth_data: self.auth_data.clone(),
            service: Rc::new(RefCell::new(service)),
        }))
    }
}

pub struct CheckLoginMiddleware<S> {
    auth_data: Rc<AuthData>,
    service: Rc<RefCell<S>>,
}

impl<S, B> Service<ServiceRequest> for CheckLoginMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;
    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        let auth_data = self.auth_data.clone();

        Box::pin(async move {
            let cookie = req.cookie("bearer");
            let path = req.path().to_string();
            log::info!("Authentication initiated for {}", &path);
            let (http_request, payload) = req.into_parts();

            // Skip authentication for unprotected URLs
            if auth_data.is_url_unprotected(&path) {
                log::debug!("Skipping authentication for unprotected URL: {}", path);
                let req = ServiceRequest::from_parts(http_request, payload);
                let http_client = HttpClient::new(reqwest::Client::new());
                req.extensions_mut().insert::<HttpClient>(http_client);
                return srv.call(req).await.map(ServiceResponse::map_into_left_body);
            }

            // Handle missing cookie
            let cookie = match cookie {
                Some(c) => c,
                None => {
                    log::warn!("Bearer token missing for path: {}", path);
                    let res = auth_data.clear_cookie("Bearer token is missing".into());
                    let res = res.map_into_right_body();
                    return Ok(ServiceResponse::new(http_request, res));
                }
            };

            // Authenticate with timeout
            // Handle authentication with proper ownership
            match auth_data.authenticate(path.clone(), cookie.value().to_string()).await {
                Err(msg) => {
                    log::warn!("Authentication failed for {}: {}", path, msg);
                    let res = auth_data.clear_cookie(msg);
                    let res = res.map_into_right_body();
                    Ok(ServiceResponse::new(http_request, res))
                },
                Ok(auth_info) => {
                    log::info!("Authentication successful for path: {}", path);
                    
                    // Create the request object
                    let req = ServiceRequest::from_parts(http_request, payload);
                    
                    // Build HTTP client with timeout settings
                    let mut new_header = header::HeaderMap::new();
                    if let Some(cookie_header) = req.headers().get(COOKIE) {
                        new_header.insert(COOKIE, cookie_header.clone());
                    }
                    
                    let client = match reqwest::Client::builder()
                        .default_headers(new_header)
                        .timeout(std::time::Duration::from_secs(5))
                        .build() 
                    {
                        Ok(c) => c,
                        Err(err) => {
                            log::error!("Failed to build HTTP client: {}", err);
                            let res = auth_data.clear_cookie("Internal server error".into());
                            let res = res.map_into_right_body();
                            return Ok(ServiceResponse::new(req.request().clone(), res));
                        }
                    };
                    
                    let http_client = HttpClient::new(client);
                    req.extensions_mut().insert::<HttpClient>(http_client);
                    req.extensions_mut().insert::<Rc<AuthInfo>>(Rc::new(auth_info));
                    srv.call(req).await.map(ServiceResponse::map_into_left_body)
                }
            }
        })
    }
}
