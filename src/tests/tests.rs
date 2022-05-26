use actix_web::{middleware, App, test, http::header};
use log::debug;
use super::simulate_standalone_server;
use httpmock::prelude::*;
use crate::{AuthenticateMiddlewareFactory, AuthData, HttpClient};
use super::utils::{initialise_logging, config, get_login_coockie};
use serde_json::json;
use std::sync::Arc;

const ALLOWED_URLS: [&str; 1] = ["/master-permission/permission/get/permission-codes/"];

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
}




/// test to check httpclient
#[actix_rt::test]
async fn test_statefull_httpclient_middleware() {

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
    let allowed_urls: Arc<Vec<String>> = Arc::new(allowed_urls);

    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(AuthenticateMiddlewareFactory::new(AuthData::new(Arc::clone(&allowed_urls))))
            .configure(config)
    ).await;

    let req = test::TestRequest::get()
        .uri("/storeservice/test/httpclient/")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .insert_header((header::COOKIE, get_login_coockie()))
        .send_request(&mut app)
        .await;
    assert_eq!(req.status(), 200);
}
