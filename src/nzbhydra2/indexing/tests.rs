use std::time::{Duration, SystemTime, UNIX_EPOCH};

use lemonfiber_fixtures::http::{Answer, Fake};
use lemonfiber_ports::http::Method;
use lemonfiber_ports::service::{Failure, IndexerUse, Indexers as _, Limits};

use super::super::tests::{nzbhydra2, presented};
use super::{failed, AccessStat};

const NOW: u64 = 1_792_886_400;

const DAY: Duration = Duration::from_secs(86_400);

const STATUSES: &str = r#"[
  {"indexer":"busy","state":"ENABLED","level":0,"disabledUntil":null,"apiResetTime":1792900000.5,
   "downloadResetTime":1792890000.0,"apiHits":100,"apiHitLimit":100,"downloadHits":10,"downloadHitLimit":10},
  {"indexer":"rested","state":"DISABLED_SYSTEM_TEMPORARY","level":1,"disabledUntil":1893456000.0,
   "apiResetTime":null,"downloadResetTime":null,"apiHits":3,"apiHitLimit":null,"downloadHits":0,"downloadHitLimit":null},
  {"indexer":"quiet","state":"DISABLED_USER","disabledUntil":1893456000.0}
]"#;

const STATS: &str = r#"{
  "indexerApiAccessStats":[
    {"indexerName":"busy","percentSuccessful":90.0,"percentConnectionError":10.0,"averageAccessesPerDay":100.0},
    {"indexerName":"rested","percentSuccessful":null,"percentConnectionError":100.0,"averageAccessesPerDay":3.0},
    {"indexerName":"quiet","percentSuccessful":null,"percentConnectionError":null,"averageAccessesPerDay":null}
  ],
  "successfulDownloadsPerIndexer":[{"indexerName":"busy","countAll":10,"countSuccessful":8,"countError":2}]
}"#;

fn at(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

#[tokio::test]
async fn every_indexer_is_answered_with_its_use_over_the_last_day_and_what_it_allows() {
    let fake = Fake::in_turn(vec![
        Answer::reply(200, STATUSES),
        Answer::reply(200, STATS),
    ]);
    let used = nzbhydra2(&fake).indexers(at(NOW)).await;
    let limits = Limits {
        queries: Some(100),
        grabs: Some(10),
        window: DAY,
    };
    assert_eq!(
        used.ok(),
        Some(vec![
            IndexerUse {
                name: "busy".to_owned(),
                enabled: true,
                queries: 100,
                failed_queries: 10,
                grabs: 10,
                failed_grabs: 2,
                rested_until: None,
                limits: Some(limits),
                searched_from: Some(at(1_792_900_000) - DAY),
                grabbed_from: Some(at(1_792_890_000) - DAY),
            },
            IndexerUse {
                name: "rested".to_owned(),
                enabled: false,
                queries: 3,
                failed_queries: 3,
                grabs: 0,
                failed_grabs: 0,
                rested_until: Some("2030-01-01T00:00:00Z".to_owned()),
                limits: None,
                searched_from: None,
                grabbed_from: None,
            },
            IndexerUse {
                name: "quiet".to_owned(),
                enabled: false,
                queries: 0,
                failed_queries: 0,
                grabs: 0,
                failed_grabs: 0,
                rested_until: Some("2030-01-01T00:00:00Z".to_owned()),
                limits: None,
                searched_from: None,
                grabbed_from: None,
            },
        ])
    );
    let requests = fake.requests();
    assert_eq!(
        requests
            .iter()
            .map(|one| (one.method, one.url.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                Method::Get,
                "http://nzbhydra2:5076/internalapi/indexerstatuses"
            ),
            (Method::Post, "http://nzbhydra2:5076/internalapi/stats"),
        ]
    );
    assert!(requests.iter().all(|one| one
        .headers
        .iter()
        .any(|(name, value)| name == "Authorization" && *value == presented())));
    let asked: Option<serde_json::Value> = requests
        .get(1)
        .and_then(|one| one.body.as_deref())
        .and_then(|body| serde_json::from_str(body).ok());
    assert_eq!(
        asked,
        Some(serde_json::json!({
            "after": "2026-10-24T00:00:00Z",
            "before": "2026-10-25T00:00:00Z",
            "includeDisabled": true,
            "indexerApiAccessStats": true,
            "successfulDownloadsPerIndexer": true,
        }))
    );
}

#[tokio::test]
async fn stats_nzbhydra2_leaves_empty_count_no_failure() {
    let fake = Fake::in_turn(vec![
        Answer::reply(200, r#"[{"indexer":"one","state":"ENABLED","apiHits":4}]"#),
        Answer::reply(
            200,
            r#"{"indexerApiAccessStats":null,"successfulDownloadsPerIndexer":null}"#,
        ),
    ]);
    let used = nzbhydra2(&fake).indexers(at(NOW)).await.unwrap_or_default();
    assert_eq!(
        used.iter()
            .map(|one| (one.queries, one.failed_queries, one.failed_grabs))
            .collect::<Vec<_>>(),
        [(4, 0, 0)]
    );
    let early = Fake::in_turn(vec![Answer::reply(200, "[]"), Answer::reply(200, "{}")]);
    assert!(nzbhydra2(&early).indexers(UNIX_EPOCH).await.is_ok());
}

#[tokio::test]
async fn an_nzbhydra2_that_refuses_this_administrator_or_answers_unreadably_is_a_failure() {
    let refused = Fake::in_turn(vec![Answer::reply(401, "")]);
    assert!(matches!(
        nzbhydra2(&refused).indexers(at(NOW)).await,
        Err(Failure::Unauthorised { .. })
    ));
    let unread = Fake::in_turn(vec![Answer::reply(200, "[]"), Answer::reply(200, "nope")]);
    assert!(matches!(
        nzbhydra2(&unread).indexers(at(NOW)).await,
        Err(Failure::Refused { .. })
    ));
}

#[test]
fn the_searches_that_failed_are_the_share_nzbhydra2_did_not_answer() {
    let stat = |percent: Option<f64>, accessed: Option<f64>| AccessStat {
        indexer_name: "one".to_owned(),
        percent_successful: percent,
        average_accesses_per_day: accessed,
    };
    assert_eq!(failed(3, Some(&stat(Some(50.0), Some(3.0)))), 1);
    assert_eq!(failed(3, Some(&stat(None, Some(3.0)))), 3);
    assert_eq!(failed(3, Some(&stat(Some(250.0), Some(3.0)))), 0);
    assert_eq!(failed(3, Some(&stat(Some(-5.0), Some(3.0)))), 3);
    assert_eq!(failed(3, Some(&stat(Some(50.0), None))), 0);
    assert_eq!(failed(3, None), 0);
    assert_eq!(failed(0, Some(&stat(Some(0.0), Some(1.0)))), 0);
}
