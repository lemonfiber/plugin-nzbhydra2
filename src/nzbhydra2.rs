//! NZBHydra2, the Usenet indexer aggregator, asked through its own API with the
//! administrator lemonfiber minted for this adapter.

use std::sync::Arc;

use base64::Engine as _;
use lemonfiber_adapter_kit::credential::Holding;
use lemonfiber_adapter_kit::endpoint::Endpoint;
use lemonfiber_ports::http::{Http, Method, Request, Response, JSON};
use lemonfiber_ports::service::Failure;
use serde::Deserialize;
use serde_json::Value;

mod indexing;
mod owning;

/// What every failure is blamed on.
pub const SERVICE: &str = "nzbhydra2";

const ACCESS: &str = "/internalapi/userinfos";

const CONFIG: &str = "/internalapi/config";

const RESTART: &str = "/internalapi/control/restart";

const AUTHORIZATION: &str = "Authorization";

const CONTENT_TYPE: &str = "Content-Type";

const NO_CREDENTIAL: &str = "this adapter holds no administrator for NZBHydra2";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Access {
    auth_configured: bool,
}

/// A client for one NZBHydra2, presenting the administrator `username` and `password`
/// supply wherever a call is made as that administrator.
pub struct Nzbhydra2 {
    endpoint: Endpoint,
    username: Arc<dyn Holding>,
    password: Arc<dyn Holding>,
}

impl Nzbhydra2 {
    /// The NZBHydra2 reached at `base` over `http`, asked as the administrator `username`
    /// and `password` supply.
    #[must_use]
    pub fn new(
        http: Arc<dyn Http>,
        base: impl Into<String>,
        username: Arc<dyn Holding>,
        password: Arc<dyn Holding>,
    ) -> Self {
        Self {
            endpoint: Endpoint::new(http, base, SERVICE),
            username,
            password,
        }
    }

    /// Whether NZBHydra2 answers how it is guarded, which it answers to anybody.
    pub async fn answers(&self) -> bool {
        self.endpoint
            .send(&self.request(Method::Get, ACCESS, None, None))
            .await
            .is_ok_and(|response| response.is_success())
    }

    fn admin(&self) -> Result<(String, String), Failure> {
        let held = |holding: &Arc<dyn Holding>| {
            holding
                .key()
                .map_err(|why| self.endpoint.refused(&format!("{NO_CREDENTIAL}: {why}")))
        };
        Ok((held(&self.username)?, held(&self.password)?))
    }

    fn presented(&self) -> Result<String, Failure> {
        let (username, password) = self.admin()?;
        let pair = format!("{username}:{password}");
        Ok(format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(pair)
        ))
    }

    fn request(
        &self,
        method: Method,
        path: &str,
        presenting: Option<String>,
        body: Option<String>,
    ) -> Request {
        let mut headers = Vec::new();
        if let Some(presented) = presenting {
            headers.push((AUTHORIZATION.to_owned(), presented));
        }
        if body.is_some() {
            headers.push((CONTENT_TYPE.to_owned(), JSON.to_owned()));
        }
        Request {
            method,
            url: self.endpoint.url(path),
            headers,
            body,
            pinned: None,
        }
    }

    async fn access(&self) -> Result<bool, Failure> {
        let response = self
            .endpoint
            .send(&self.request(Method::Get, ACCESS, None, None))
            .await?;
        let access: Access = self
            .endpoint
            .decode(&response, "how NZBHydra2 is guarded could not be read")?;
        Ok(access.auth_configured)
    }

    async fn configuration(&self, presenting: Option<String>) -> Result<Response, Failure> {
        self.endpoint
            .send(&self.request(Method::Get, CONFIG, presenting, None))
            .await
    }

    async fn read_as_admin<T: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<String>,
        what: &str,
    ) -> Result<T, Failure> {
        let response = self
            .endpoint
            .send(&self.request(method, path, Some(self.presented()?), body))
            .await?;
        self.endpoint.decode(&response, what)
    }

    async fn restart(&self) -> Result<(), Failure> {
        let response = self
            .endpoint
            .send(&self.request(Method::Get, RESTART, None, None))
            .await?;
        self.endpoint.expect_success(&response)
    }
}

fn held_users(config: &Value) -> Vec<&str> {
    config
        .pointer("/auth/users")
        .and_then(Value::as_array)
        .map(|users| {
            users
                .iter()
                .filter_map(|user| user.get("username").and_then(Value::as_str))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
