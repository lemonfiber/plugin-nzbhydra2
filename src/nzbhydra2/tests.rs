use std::sync::Arc;

use lemonfiber_fixtures::http::{Answer, Fake};
use lemonfiber_ports::http::Method;
use lemonfiber_ports::service::{Failure, Owner, Ownership as _};

use super::Nzbhydra2;

const USERNAME: &str = "abcdefgh";

const PASSWORD: &str = "ijklmnopqrstuvwx";

const OPEN: &str = r#"{"authConfigured":false}"#;

const GUARDED: &str = r#"{"authConfigured":true}"#;

const UNGUARDED_CONFIG: &str = r#"{"auth":{"authType":"NONE","users":[]},"indexers":[]}"#;

struct Missing;

impl lemonfiber_adapter_kit::credential::Holding for Missing {
    fn key(&self) -> Result<String, String> {
        Err("upstream.json could not be read: not found".to_owned())
    }
}

pub(super) fn nzbhydra2(fake: &Arc<Fake>) -> Nzbhydra2 {
    Nzbhydra2::new(
        fake.clone(),
        "http://nzbhydra2:5076",
        Arc::new(USERNAME.to_owned()),
        Arc::new(PASSWORD.to_owned()),
    )
}

pub(super) fn presented() -> String {
    use base64::Engine as _;
    format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(format!("{USERNAME}:{PASSWORD}"))
    )
}

fn authorization(request: &lemonfiber_ports::http::Request) -> Option<&str> {
    request
        .headers
        .iter()
        .find(|(name, _)| name == "Authorization")
        .map(|(_, value)| value.as_str())
}

fn asked(fake: &Fake) -> Vec<(Method, String)> {
    fake.requests()
        .into_iter()
        .map(|one| {
            (
                one.method,
                one.url
                    .trim_start_matches("http://nzbhydra2:5076")
                    .to_owned(),
            )
        })
        .collect()
}

#[tokio::test]
async fn an_nzbhydra2_with_no_authentication_is_open_to_anybody() {
    let fake = Fake::in_turn(vec![Answer::reply(200, OPEN)]);
    assert!(matches!(nzbhydra2(&fake).owner().await, Ok(Owner::Open)));
    assert_eq!(
        asked(&fake),
        [(Method::Get, "/internalapi/userinfos".to_owned())]
    );
    assert!(fake
        .requests()
        .iter()
        .all(|one| authorization(one).is_none()));
}

#[tokio::test]
async fn authentication_saved_and_not_yet_in_force_is_settling_where_it_names_this_administrator_alone(
) {
    let ours = format!(
        r#"{{"auth":{{"users":[{{"username":"{USERNAME}","password":"***UNCHANGED***"}}]}}}}"#
    );
    let fake = Fake::in_turn(vec![Answer::reply(200, GUARDED), Answer::reply(200, ours)]);
    assert!(matches!(
        nzbhydra2(&fake).owner().await,
        Ok(Owner::Settling)
    ));
    for users in [
        r#"{"auth":{"users":[{"username":"somebody"}]}}"#,
        r#"{"auth":{"users":[{"username":"abcdefgh"},{"username":"somebody"}]}}"#,
        r#"{"auth":{}}"#,
    ] {
        let fake = Fake::in_turn(vec![Answer::reply(200, GUARDED), Answer::reply(200, users)]);
        assert!(
            matches!(nzbhydra2(&fake).owner().await, Ok(Owner::Theirs)),
            "{users}"
        );
    }
}

#[tokio::test]
async fn authentication_in_force_is_ours_where_this_administrator_is_let_in_and_theirs_where_not() {
    let fake = Fake::in_turn(vec![
        Answer::reply(200, GUARDED),
        Answer::reply(401, ""),
        Answer::reply(200, UNGUARDED_CONFIG),
    ]);
    assert!(matches!(nzbhydra2(&fake).owner().await, Ok(Owner::Ours)));
    let requests = fake.requests();
    assert_eq!(
        requests.iter().map(authorization).collect::<Vec<_>>(),
        [None, None, Some(presented().as_str())]
    );
    for refused in [401, 403] {
        let fake = Fake::in_turn(vec![
            Answer::reply(200, GUARDED),
            Answer::reply(403, ""),
            Answer::reply(refused, ""),
        ]);
        assert!(matches!(nzbhydra2(&fake).owner().await, Ok(Owner::Theirs)));
    }
}

#[tokio::test]
async fn an_answer_that_is_neither_the_configuration_nor_a_refusal_is_a_failure() {
    let exposed = Fake::in_turn(vec![Answer::reply(200, GUARDED), Answer::reply(500, "")]);
    assert!(matches!(
        nzbhydra2(&exposed).owner().await,
        Err(Failure::Refused { .. })
    ));
    let ours = Fake::in_turn(vec![
        Answer::reply(200, GUARDED),
        Answer::reply(401, ""),
        Answer::reply(500, ""),
    ]);
    assert!(matches!(
        nzbhydra2(&ours).owner().await,
        Err(Failure::Refused { .. })
    ));
    let unread = Fake::in_turn(vec![
        Answer::reply(200, GUARDED),
        Answer::reply(200, "not json"),
    ]);
    assert!(matches!(
        nzbhydra2(&unread).owner().await,
        Err(Failure::Refused { .. })
    ));
    assert!(matches!(
        nzbhydra2(&Fake::silent()).owner().await,
        Err(Failure::Unavailable { .. })
    ));
}

#[tokio::test]
async fn an_open_nzbhydra2_is_claimed_for_this_administrator_alone_and_restarted() {
    let fake = Fake::in_turn(vec![
        Answer::reply(200, OPEN),
        Answer::reply(200, UNGUARDED_CONFIG),
        Answer::reply(
            200,
            r#"{"ok":true,"restartNeeded":true,"errorMessages":[]}"#,
        ),
        Answer::reply(200, ""),
    ]);
    assert!(matches!(
        nzbhydra2(&fake).claim().await,
        Ok(Owner::Settling)
    ));
    assert_eq!(
        asked(&fake),
        [
            (Method::Get, "/internalapi/userinfos".to_owned()),
            (Method::Get, "/internalapi/config".to_owned()),
            (Method::Put, "/internalapi/config".to_owned()),
            (Method::Get, "/internalapi/control/restart".to_owned()),
        ]
    );
    let requests = fake.requests();
    let saved: Option<serde_json::Value> = requests
        .get(2)
        .and_then(|one| one.body.as_deref())
        .and_then(|body| serde_json::from_str(body).ok());
    assert_eq!(
        saved,
        Some(serde_json::json!({
            "auth": {
                "authType": "BASIC",
                "restrictAdmin": true,
                "restrictSearch": true,
                "restrictStats": true,
                "restrictDetailsDl": true,
                "restrictIndexerSelection": true,
                "users": [{
                    "username": USERNAME,
                    "password": PASSWORD,
                    "maySeeAdmin": true,
                    "maySeeStats": true,
                    "maySeeDetailsDl": true,
                    "showIndexerSelection": true,
                }],
            },
            "indexers": [],
        }))
    );
    assert!(requests.get(2).is_some_and(|one| one
        .headers
        .iter()
        .any(|(name, value)| name == "Content-Type" && value == "application/json")));
    assert!(requests.iter().all(|one| authorization(one).is_none()));
}

#[tokio::test]
async fn a_claim_nzbhydra2_refuses_or_cannot_hold_changes_nothing_and_restarts_nothing() {
    let refused = Fake::in_turn(vec![
        Answer::reply(200, OPEN),
        Answer::reply(200, UNGUARDED_CONFIG),
        Answer::reply(200, r#"{"ok":false,"errorMessages":["one","two"]}"#),
    ]);
    assert!(matches!(
        nzbhydra2(&refused).claim().await,
        Err(Failure::Refused { detail, .. }) if detail == "one; two"
    ));
    assert_eq!(refused.requests().len(), 3);
    let rejected = Fake::in_turn(vec![
        Answer::reply(200, OPEN),
        Answer::reply(200, UNGUARDED_CONFIG),
        Answer::reply(400, "bad"),
    ]);
    assert!(matches!(
        nzbhydra2(&rejected).claim().await,
        Err(Failure::Refused { .. })
    ));
    assert_eq!(rejected.requests().len(), 3);
    let sectionless = Fake::in_turn(vec![
        Answer::reply(200, OPEN),
        Answer::reply(200, r#"{"indexers":[]}"#),
    ]);
    assert!(matches!(
        nzbhydra2(&sectionless).claim().await,
        Err(Failure::Refused { .. })
    ));
    assert_eq!(sectionless.requests().len(), 2);
    let unrestarted = Fake::in_turn(vec![
        Answer::reply(200, OPEN),
        Answer::reply(200, UNGUARDED_CONFIG),
        Answer::reply(200, r#"{"ok":true}"#),
        Answer::reply(500, ""),
    ]);
    assert!(nzbhydra2(&unrestarted).claim().await.is_err());
}

#[tokio::test]
async fn a_claim_already_saved_for_this_administrator_is_restarted_again_and_nobody_else_s_is_touched(
) {
    let ours = format!(r#"{{"auth":{{"users":[{{"username":"{USERNAME}"}}]}}}}"#);
    let settling = Fake::in_turn(vec![
        Answer::reply(200, GUARDED),
        Answer::reply(200, ours),
        Answer::reply(200, ""),
    ]);
    assert!(matches!(
        nzbhydra2(&settling).claim().await,
        Ok(Owner::Settling)
    ));
    assert_eq!(
        asked(&settling).last(),
        Some(&(Method::Get, "/internalapi/control/restart".to_owned()))
    );
    let held = Fake::in_turn(vec![
        Answer::reply(200, GUARDED),
        Answer::reply(401, ""),
        Answer::reply(200, UNGUARDED_CONFIG),
    ]);
    assert!(matches!(nzbhydra2(&held).claim().await, Ok(Owner::Ours)));
    assert_eq!(held.requests().len(), 3);
    let theirs = Fake::in_turn(vec![
        Answer::reply(200, GUARDED),
        Answer::reply(401, ""),
        Answer::reply(401, ""),
    ]);
    assert!(matches!(
        nzbhydra2(&theirs).claim().await,
        Ok(Owner::Theirs)
    ));
    assert!(asked(&theirs)
        .iter()
        .all(|(method, path)| *method == Method::Get && path != "/internalapi/control/restart"));
}

#[tokio::test]
async fn without_the_administrator_nothing_is_claimed_and_the_refusal_names_no_part_of_one() {
    let fake = Fake::in_turn(vec![
        Answer::reply(200, OPEN),
        Answer::reply(200, UNGUARDED_CONFIG),
    ]);
    let headless = Nzbhydra2::new(
        fake.clone(),
        "http://nzbhydra2:5076",
        Arc::new(USERNAME.to_owned()),
        Arc::new(Missing),
    );
    assert!(matches!(
        headless.claim().await,
        Err(Failure::Refused { detail, .. }) if !detail.contains(USERNAME)
    ));
    assert!(asked(&fake)
        .iter()
        .all(|(method, _)| *method == Method::Get));
}

#[tokio::test]
async fn it_answers_while_nzbhydra2_says_how_it_is_guarded() {
    assert!(
        nzbhydra2(&Fake::always(Answer::reply(200, OPEN)))
            .answers()
            .await
    );
    assert!(
        !nzbhydra2(&Fake::always(Answer::reply(503, "")))
            .answers()
            .await
    );
    assert!(!nzbhydra2(&Fake::silent()).answers().await);
}
