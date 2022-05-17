# Authmiddleware
An Middleware for authentication of users in Actix-web library.


Example :1

/// This export is where token sent for verification and authentication
export TOKENSERVICE_URL="http://token-service:5000";

```

use actix_web::{HttpResponse, middleware, App, test, http::header};
use crate::{AuthenticateMiddlewareFactory, AuthData};
use serde_json::json;


    let ALLOWED_URLS: [String; 1] = ["/master-permission/permission/get/permission-codes/".to_string()];
    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(AuthenticateMiddlewareFactory::new(AuthData::new(ALLOWED_URLS.clone())))
            .configure(config)
    ).await;

    let req = test::TestRequest::get()
        .uri("/storeservice/health/")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .insert_header((header::COOKIE, get_login_coockie()))
        .send_request(&mut app)
        .await;
    assert_eq!(req.status(), 200);
```
