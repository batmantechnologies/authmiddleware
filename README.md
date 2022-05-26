# Authmiddleware
An Middleware for authentication of users in Actix-web library.


Example :1

This export is where token sent for verification and authentication

```
export TOKENSERVICE_URL="http://token-service:5000";

```

```

use actix_web::{middleware, App, test, http::header};
use log::debug;
use super::simulate_standalone_server;
use httpmock::prelude::*;
use crate::{AuthenticateMiddlewareFactory, AuthData};
use super::utils::{initialise_logging, config, get_login_coockie};
use serde_json::json;
use std::sync::Arc;

    // Start HTTP server
    let allowed_urls = ALLOWED_URLS.into_iter().map(|val|{val.to_string()}).collect();
    let allowed_urls: Arc<Vec<String>> = Arc::new(allowed_urls);

    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(AuthenticateMiddlewareFactory::new(AuthData::new(Arc::clone(&allowed_urls))))
            .configure(config)
    ).await;

    let req = test::TestRequest::get()
        .uri("/storeservice/health/")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .send_request(&mut app)
        .await;
    assert_eq!(req.status(), 403);
```



THis is how we can get authenticated user details in an controller, Refer tests for this module.

```
pub async fn check_health(auth_info: Option<ReqData<Rc<AuthInfo>>>) -> Result<HttpResponse, Error> {
    let auth_info = auth_info.unwrap().get_data();
    info!("UserID: {0}, AppID: {1}", auth_info.0, auth_info.1);
    Ok(HttpResponse::Ok().json("Service is reachable"))
}

 ```
