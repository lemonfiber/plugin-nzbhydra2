//! The indexers NZBHydra2 searches and how hard each has been used; and the parts of the
//! indexer contract NZBHydra2 has no equivalent for, answered unsupported.

use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use lemonfiber_ports::http::Method;
use lemonfiber_ports::service::{
    Aggregator, Aggregators, AppSync, Application, Failure, IndexerUse, Indexers, KnownAggregator,
    Limits, RegisteredApplication,
};
use serde::Deserialize;

use super::{Nzbhydra2, SERVICE};
use crate::instant;

const STATUSES: &str = "/internalapi/indexerstatuses";

const STATS: &str = "/internalapi/stats";

const ENABLED: &str = "ENABLED";

const WINDOW: Duration = Duration::from_secs(24 * 60 * 60);

const NO_APPLICATIONS: &str =
    "NZBHydra2 is searched by applications and is told about none of them";

const NO_AGGREGATORS: &str = "NZBHydra2 searches its indexers itself and reads from no aggregator";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    indexer: String,
    state: String,
    #[serde(default)]
    disabled_until: Option<f64>,
    #[serde(default)]
    api_reset_time: Option<f64>,
    #[serde(default)]
    download_reset_time: Option<f64>,
    #[serde(default)]
    api_hits: Option<u64>,
    #[serde(default)]
    api_hit_limit: Option<u64>,
    #[serde(default)]
    download_hits: Option<u64>,
    #[serde(default)]
    download_hit_limit: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccessStat {
    indexer_name: String,
    #[serde(default)]
    percent_successful: Option<f64>,
    #[serde(default)]
    average_accesses_per_day: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadStat {
    indexer_name: String,
    #[serde(default)]
    count_error: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Stats {
    #[serde(default)]
    indexer_api_access_stats: Option<Vec<AccessStat>>,
    #[serde(default)]
    successful_downloads_per_indexer: Option<Vec<DownloadStat>>,
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "a share of searches, rounded to a whole search no larger than the searches made"
)]
fn failed(queries: u64, stat: Option<&AccessStat>) -> u64 {
    let Some(stat) = stat.filter(|stat| stat.average_accesses_per_day.is_some()) else {
        return 0;
    };
    let succeeded = stat.percent_successful.unwrap_or(0.0).clamp(0.0, 100.0);
    let answered = (queries as f64 * succeeded / 100.0).round() as u64;
    queries.saturating_sub(answered.min(queries))
}

fn window_from(reset: Option<f64>) -> Option<SystemTime> {
    instant::from_seconds(reset?)?.checked_sub(WINDOW)
}

fn used(status: Status, measured: &Stats) -> IndexerUse {
    let access = measured
        .indexer_api_access_stats
        .iter()
        .flatten()
        .find(|stat| stat.indexer_name == status.indexer);
    let failed_grabs = measured
        .successful_downloads_per_indexer
        .iter()
        .flatten()
        .find(|stat| stat.indexer_name == status.indexer)
        .map_or(0, |stat| stat.count_error);
    let queries = status.api_hits.unwrap_or(0);
    let enabled = status.state == ENABLED;
    let limits =
        (status.api_hit_limit.is_some() || status.download_hit_limit.is_some()).then_some(Limits {
            queries: status.api_hit_limit,
            grabs: status.download_hit_limit,
            window: WINDOW,
        });
    IndexerUse {
        enabled,
        queries,
        failed_queries: failed(queries, access),
        grabs: status.download_hits.unwrap_or(0),
        failed_grabs,
        rested_until: (!enabled)
            .then_some(status.disabled_until)
            .flatten()
            .and_then(instant::from_seconds)
            .and_then(instant::written),
        limits,
        searched_from: window_from(status.api_reset_time),
        grabbed_from: window_from(status.download_reset_time),
        name: status.indexer,
    }
}

#[async_trait]
impl Indexers for Nzbhydra2 {
    async fn indexers(&self, now: SystemTime) -> Result<Vec<IndexerUse>, Failure> {
        let statuses: Vec<Status> = self
            .read_as_admin(
                Method::Get,
                STATUSES,
                None,
                "the indexers NZBHydra2 searches could not be read",
            )
            .await?;
        let since = now.checked_sub(WINDOW).unwrap_or(SystemTime::UNIX_EPOCH);
        let asked = serde_json::json!({
            "after": instant::written(since),
            "before": instant::written(now),
            "includeDisabled": true,
            "indexerApiAccessStats": true,
            "successfulDownloadsPerIndexer": true,
        });
        let stats: Stats = self
            .read_as_admin(
                Method::Post,
                STATS,
                Some(asked.to_string()),
                "how NZBHydra2's indexers answered could not be read",
            )
            .await?;
        Ok(statuses
            .into_iter()
            .map(|status| used(status, &stats))
            .collect())
    }
}

fn unsupported(detail: &str) -> Failure {
    Failure::Unsupported {
        service: SERVICE.to_owned(),
        detail: detail.to_owned(),
    }
}

#[async_trait]
impl AppSync for Nzbhydra2 {
    async fn register_application(&self, _application: &Application) -> Result<(), Failure> {
        Err(unsupported(NO_APPLICATIONS))
    }

    async fn applications(&self) -> Result<Vec<RegisteredApplication>, Failure> {
        Err(unsupported(NO_APPLICATIONS))
    }

    async fn test_application(&self, _held: &RegisteredApplication) -> Result<(), Failure> {
        Err(unsupported(NO_APPLICATIONS))
    }

    async fn rekey_application(
        &self,
        _held: &RegisteredApplication,
        _key: &str,
    ) -> Result<(), Failure> {
        Err(unsupported(NO_APPLICATIONS))
    }
}

#[async_trait]
impl Aggregators for Nzbhydra2 {
    async fn aggregators(&self) -> Result<Vec<KnownAggregator>, Failure> {
        Err(unsupported(NO_AGGREGATORS))
    }

    async fn add_aggregator(&self, _aggregator: &Aggregator) -> Result<(), Failure> {
        Err(unsupported(NO_AGGREGATORS))
    }
}

#[cfg(test)]
mod tests;
