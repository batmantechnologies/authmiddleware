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
            let path = req.path().clone().to_string();
            log::info!("Authenticaiton Initiated for {}", &path);
            let (request, paylaod) = req.into_parts();

            // Skipping unprotected URLS below URL where
            if auth_data.is_url_unprotected(&path) {
                let req = ServiceRequest::from_parts(request, paylaod);
                let http_client = HttpClient::new(reqwest::Client::new());
                req.extensions_mut().insert::<HttpClient>(http_client);
                return srv.call(req).await.map(ServiceResponse::map_into_left_body);
            } else if cookie.is_none() {
                log::debug!("Bearer Token is Missing");
                let res = auth_data.clear_cookie("Bearer Token is Missing".into());
                let res = res.map_into_right_body();
                return Ok(ServiceResponse::new(request, res))
            }
            let cookie = cookie.unwrap();
            let auth_result = auth_data.authenticate(path.clone(), cookie.value().to_string()).await;
            if let Err(msg) = auth_result {
                log::debug!("Path URL Authentication Failed: {}", &path);
                let res = auth_data.clear_cookie(msg);
                let res = res.map_into_right_body();
                return Ok(ServiceResponse::new(request, res))
            } else {
                let auth_info: AuthInfo = auth_result.unwrap();
                let req = ServiceRequest::from_parts(request, paylaod);
                let mut new_header = header::HeaderMap::new();
                new_header.insert(COOKIE, req.headers().get(COOKIE).unwrap().clone());
                let client = reqwest::Client::builder()
                    .default_headers(new_header)
                    .build().unwrap();
                let http_client = HttpClient::new(client);
                req.extensions_mut().insert::<HttpClient>(http_client);
                req.extensions_mut().insert::<Rc<AuthInfo>>(Rc::new(auth_info));
                return srv.call(req).await.map(ServiceResponse::map_into_left_body);
            }
        })
    }
}
