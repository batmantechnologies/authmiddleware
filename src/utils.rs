use log;
use serde_json::json;
use crate::apicalls;
use serde::{Serialize, Deserialize};
use reqwest::{self, Client};
use std::sync::Arc;
use tokio::time;

use actix_web::{
    HttpResponse, cookie::Cookie
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    user_id: i32,
    app_id: i32,
    path: String,
    token_id: i32
}

#[derive(Clone, Debug)]
pub struct HttpClient {
    client: Client,
}

#[derive(Clone, Debug)]
pub struct AuthData {
    token_url: String,
    http_client: reqwest::Client,
    unprotected_urls: Arc<Vec<String>>,
}

impl AuthInfo {
    pub fn get_data(&self) -> (i32, i32) {
        (self.user_id.clone(), self.app_id.clone())
    }

    pub fn get_token_id(&self) -> i32 {
        self.token_id.clone()
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }
}

impl HttpClient {
    pub fn new(client: Client) -> HttpClient {
        HttpClient { client: client }
    }

    pub fn get_client(&self) -> Client {
        self.client.clone()
    }

}

impl AuthData {

    pub fn new(unprotected_urls: Arc<Vec<String>>) -> AuthData {
        AuthData {
            token_url: apicalls::get_proxy_url(),
            http_client: reqwest::Client::new(),
            unprotected_urls: unprotected_urls
        }
    }

    pub fn is_url_unprotected(&self, url: &String) -> bool {
        if self.unprotected_urls.contains(url) {
            return true
        } else {
            return false
        }
    }

    pub fn clear_cookie(&self, message: String) -> HttpResponse {
        log::debug!("Cleared coockie. as it cannot be authenticated or authorised");
        let cookie = Cookie::build("bearer","").path("/").finish();
        let mut response = HttpResponse::Forbidden().json(message);
        response.add_removal_cookie(&cookie).unwrap();
        return response
    }

    pub fn forbid_keep_cookie(&self, message: String) -> HttpResponse {
        log::debug!("Forbiding but not clearing coockie");
        let response = HttpResponse::Forbidden().json(message);
        return response
    }

    // Checks if the path has access to the requested url with timeout
    pub async fn authenticate(&self, path: String, cookie_string: String) -> Result<AuthInfo, String> {
        let url = format!("{}/token/verify-token/", self.token_url);
        
        // Create a timeout for the request
        let timeout_duration = time::Duration::from_secs(3);
        
        match time::timeout(timeout_duration, async {
            self.http_client.post(&url)
                .json(&json!({
                    "path": path,
                    "token_code": cookie_string
                }))
                .send()
                .await
        }).await {
            Ok(Ok(res)) => {
                match res.status().as_u16() {
                    200 => {
                        res.json::<AuthInfo>().await.map_err(|err| {
                            log::error!("Failed to parse auth response: {}", err);
                            "Internal server error".to_string()
                        })
                    },
                    _ => {
                        let error = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        log::error!("Token verification failed: {}", error);
                        Err(error)
                    }
                }
            },
            Ok(Err(err)) => {
                log::error!("Request to token service failed: {}", err);
                Err("Failed to connect to authentication service".to_string())
            },
            Err(_) => {
                log::error!("Token verification request timed out");
                Err("Authentication service timeout".to_string())
            }
        }
    }
}
