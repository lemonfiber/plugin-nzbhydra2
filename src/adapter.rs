//! What this adapter says it is, and the kit that serves its contracts.

use std::sync::Arc;

use lemonfiber_adapter_kit::{Kit, Readiness};
use lemonfiber_contract::adapter::{About, Release};
use lemonfiber_contract::capabilities::{indexer, owner};

use crate::nzbhydra2::Nzbhydra2;

/// Where NZBHydra2 answers, by its service id and the port it listens on.
pub const ADDRESS: &str = "http://nzbhydra2:5076";

/// The upstream this adapter fronts, as its project names itself.
pub const UPSTREAM: &str = "NZBHydra2";

/// The NZBHydra2 release this adapter's recordings were taken from.
pub const RELEASE: &str = "v8.9.0-ls104";

/// The digest of the image that release's recordings were taken from.
pub const DIGEST: &str = "sha256:3cdcea6fc97861bb30ef2551884f1c371e08a884c2f75230c7a3a2e14c5f0516";

/// What this adapter says it is: the contracts it speaks and the release it supports.
#[must_use]
pub fn about() -> About {
    About {
        speaks: vec![
            lemonfiber_contract::spoken(owner::claim::CAPABILITY, owner::claim::MAJOR),
            lemonfiber_contract::spoken(indexer::search::CAPABILITY, indexer::search::MAJOR),
        ],
        upstream: UPSTREAM.to_owned(),
        releases: vec![Release {
            version: RELEASE.to_owned(),
            digest: DIGEST.to_owned(),
        }],
    }
}

/// The contracts, served from `nzbhydra2` under `key`, ready while NZBHydra2 answers.
#[must_use]
pub fn kit(key: String, nzbhydra2: Arc<Nzbhydra2>) -> Kit {
    let asked = Arc::clone(&nzbhydra2);
    let ready: Readiness = Arc::new(move || {
        let asked = Arc::clone(&asked);
        Box::pin(async move { asked.answers().await })
    });
    Kit::new(key, about())
        .ready_when(ready)
        .serving(owner::claim::served(Arc::clone(&nzbhydra2)))
        .serving(indexer::search::served(nzbhydra2))
}

#[cfg(test)]
mod tests;
