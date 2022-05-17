use log::{info};
use std::env;
use actix_web::{web, Error, HttpResponse, middleware, guard,
                App, test, http::header, web::ReqData};
use super::simulate_standalone_server;
use httpmock::prelude::*;
use crate::{AuthenticateMiddlewareFactory, AuthData, AuthInfo};
use serde_json::json;
use std::rc::Rc;

const ALLOWED_URLS: [&str; 1] = ["/master-permission/permission/get/permission-codes/"];
/// This function initiates logging
fn initialise_logging() {
    env::set_var("TOKENSERVICE_URL", "http://localhost:5000");
    let _result = env_logger::try_init();
    dotenv::dotenv().ok();
}

pub async fn check_health(auth_info: Option<ReqData<Rc<AuthInfo>>>) -> Result<HttpResponse, Error> {
    let auth_info = auth_info.unwrap().get_data();
    info!("UserID: {0}, AppID: {1}", auth_info.0, auth_info.1);
    Ok(HttpResponse::Ok().json("Service is reachable"))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/storeservice/health/")
            .name("health_check")
            .guard(guard::Header("content-type", "application/json"))
            .route(web::get().to(check_health))
    );
}

// This is intended to return the login cookie
fn get_login_coockie() -> String {
    let coockie_string: String = "bearer=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiIsImtpZCI6IjIifQ.eyJ1c2VyX2lkIjoyMCwiYXBwX2lkIjoyLCJhdXRob3JpemF0aW9uX2lkIjoxOSwidG9rZW5faWQiOjIwMjUsInBlcm1pc3Npb25fY29kZXMiOltdLCJleHAiOjE2ODQyMTgzNDF9.4tgyEYVqUZA1wSQ3iPjCozesB1gqi41VxtB8dcsF__i6oWtvncwOAlYEJfTHC0wuElAPc956u63rCCyIPt6UCA; Path=/;".to_string();
    coockie_string
}

/// Hit Store service through middleware
#[actix_rt::test]
async fn hit_store_service_middleware_test() {

    initialise_logging();
    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Instead of creating a new MockServer using connect_from_env_async(), we connect by
    // reading the host and port from the environment (HTTPMOCK_HOST / HTTPMOCK_PORT) or
    // falling back to defaults (localhost on port 5000)
    let server = MockServer::connect_from_env_async().await;

    let _search_mock = server
        .mock_async(|when, then| {
            when.method(POST)
                .path_contains("/token/verify-token/");
            then.header("content-type", "application/json")
                .json_body(json!({"app_id": 2_u32 , "user_id": 2_u32}))
                .status(200);
        }).await;


    // Start HTTP server
    let allowed_urls = ALLOWED_URLS.into_iter().map(|val|{val.to_string()}).collect();
    let allowed_urls: Rc<Vec<String>> = Rc::new(allowed_urls);

    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(AuthenticateMiddlewareFactory::new(AuthData::new(Rc::clone(&allowed_urls))))
            .configure(config)
    ).await;

    let req = test::TestRequest::get()
        .uri("/storeservice/health/")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .insert_header((header::COOKIE, get_login_coockie()))
        .send_request(&mut app)
        .await;
    assert_eq!(req.status(), 200);
}

/// Hit Store service through middleware and fail to reach it
#[actix_rt::test]
async fn hit_store_service_without_auth_coockie_test() {

    initialise_logging();
    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Instead of creating a new MockServer using connect_from_env_async(), we connect by
    // reading the host and port from the environment (HTTPMOCK_HOST / HTTPMOCK_PORT) or
    // falling back to defaults (localhost on port 5000)
    let server = MockServer::connect_from_env_async().await;

    let _search_mock = server
        .mock_async(|when, then| {
            when.method(POST)
                .path_contains("/token/verify-token/");
            then.header("content-type", "application/json")
                .json_body(json!({"app_id": 2_u32 , "user_id": 2_u32}))
                .status(200);
        }).await;

    // Start HTTP server
    let allowed_urls = ALLOWED_URLS.into_iter().map(|val|{val.to_string()}).collect();
    let allowed_urls: Rc<Vec<String>> = Rc::new(allowed_urls);

    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(AuthenticateMiddlewareFactory::new(AuthData::new(Rc::clone(&allowed_urls))))
            .configure(config)
    ).await;

    let req = test::TestRequest::get()
        .uri("/storeservice/health/")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .send_request(&mut app)
        .await;
    assert_eq!(req.status(), 403);
}
