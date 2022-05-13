use log::{info, debug};
use serde_json::json;
use crate::apicalls::get_token_url;
use serde::{Serialize, Deserialize};
use reqwest;

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse, cookie::Cookie
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    user_id: i32,
    app_id: i32,
}

#[derive(Clone, Debug)]
pub struct AuthData {
    token_url: String,
    http_client: reqwest::Client,
}

impl AuthInfo {
    pub fn get_data(&self) -> (i32, i32) {
        (self.user_id.clone(), self.app_id.clone())
    }
}

impl AuthData {

    pub fn new() -> AuthData {

        AuthData {
            token_url: get_token_url(),
            http_client: reqwest::Client::new(),
        }
    }

    pub fn clear_cookie(&self, message: String) -> HttpResponse {

        let cookie = Cookie::build("bearer","").path("/").finish();
        let mut response = HttpResponse::Forbidden().json(message);
        response.add_removal_cookie(&cookie);
        return response
    }

    pub async fn authenticate(&self, path: String, cookie_string: String) -> Result<AuthInfo, String> {

        let res = self.http_client.clone().post(self.token_url.clone()+"/token/verify-token/")
            .json(&json!({
                "path": path,
                "token_code": cookie_string
            }))
            .send()
            .await.unwrap();

        match res.status().as_u16() {
            200 => {
                let auth_info = res.json::<AuthInfo>().await.unwrap();
                Ok(auth_info)
            },
            _   => Err(res.json::<String>().await.unwrap())
        }

    }
}
