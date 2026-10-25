//! Who holds NZBHydra2, and claiming it: authentication turned on for the administrator
//! lemonfiber minted, every part guarded, and the service restarted so it takes effect.

use async_trait::async_trait;
use lemonfiber_ports::http::Method;
use lemonfiber_ports::service::{Failure, Owner, Ownership};
use serde::Deserialize;
use serde_json::Value;

use super::{held_users, Nzbhydra2, CONFIG};

const BASIC: &str = "BASIC";

const RESTRICTED: [&str; 5] = [
    "restrictAdmin",
    "restrictSearch",
    "restrictStats",
    "restrictDetailsDl",
    "restrictIndexerSelection",
];

const NO_AUTH_SECTION: &str = "NZBHydra2's configuration holds no authentication section";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Saved {
    ok: bool,
    #[serde(default)]
    error_messages: Vec<String>,
}

impl Nzbhydra2 {
    async fn guard(&self, mut config: Value) -> Result<(), Failure> {
        let (username, password) = self.admin()?;
        let auth = config
            .get_mut("auth")
            .and_then(Value::as_object_mut)
            .ok_or_else(|| self.endpoint.refused(NO_AUTH_SECTION))?;
        auth.insert("authType".to_owned(), Value::from(BASIC));
        for restricted in RESTRICTED {
            auth.insert(restricted.to_owned(), Value::Bool(true));
        }
        auth.insert(
            "users".to_owned(),
            serde_json::json!([{
                "username": username,
                "password": password,
                "maySeeAdmin": true,
                "maySeeStats": true,
                "maySeeDetailsDl": true,
                "showIndexerSelection": true,
            }]),
        );
        let response = self
            .endpoint
            .send(&self.request(Method::Put, CONFIG, None, Some(config.to_string())))
            .await?;
        let saved: Saved = self.endpoint.decode(
            &response,
            "what NZBHydra2 made of its guarded configuration could not be read",
        )?;
        if saved.ok {
            Ok(())
        } else {
            Err(self.endpoint.refused(&saved.error_messages.join("; ")))
        }
    }
}

#[async_trait]
impl Ownership for Nzbhydra2 {
    async fn owner(&self) -> Result<Owner, Failure> {
        if !self.access().await? {
            return Ok(Owner::Open);
        }
        let exposed = self.configuration(None).await?;
        if exposed.is_success() {
            let config: Value = self
                .endpoint
                .decode(&exposed, "NZBHydra2's configuration could not be read")?;
            let (username, _) = self.admin()?;
            return Ok(if held_users(&config) == [username.as_str()] {
                Owner::Settling
            } else {
                Owner::Theirs
            });
        }
        if !matches!(exposed.status, 401 | 403) {
            return Err(self.endpoint.refusal(&exposed));
        }
        let ours = self.configuration(Some(self.presented()?)).await?;
        match ours.status {
            401 | 403 => Ok(Owner::Theirs),
            _ if ours.is_success() => Ok(Owner::Ours),
            _ => Err(self.endpoint.refusal(&ours)),
        }
    }

    async fn claim(&self) -> Result<Owner, Failure> {
        match self.owner().await? {
            Owner::Open => {
                let unguarded = self.configuration(None).await?;
                let config: Value = self
                    .endpoint
                    .decode(&unguarded, "NZBHydra2's configuration could not be read")?;
                self.guard(config).await?;
                self.restart().await?;
                Ok(Owner::Settling)
            }
            Owner::Settling => {
                self.restart().await?;
                Ok(Owner::Settling)
            }
            held => Ok(held),
        }
    }
}
