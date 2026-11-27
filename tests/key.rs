//! NZBHydra2's Newznab API key is the one lemonfiber mints, written into its starting
//! `nzbhydra.yml` and handed to no adapter.

use toml::{Table, Value};

/// The secret NZBHydra2 answers its Newznab API to.
const KEY: &str = "api-key";

/// The service it is the key of.
const SERVICE: &str = "nzbhydra2";

/// Where NZBHydra2 reads it before it first starts.
const STARTING: &str = "config/nzbhydra2/nzbhydra.yml";

fn read(name: &str) -> String {
    std::fs::read_to_string(format!("{}/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap_or_default()
}

fn manifest() -> Table {
    read("plugin.toml").parse().unwrap_or_default()
}

fn listed<'a>(manifest: &'a Table, name: &str) -> Vec<&'a Table> {
    manifest
        .get(name)
        .and_then(Value::as_array)
        .map(|list| list.iter().filter_map(Value::as_table).collect())
        .unwrap_or_default()
}

fn text<'a>(table: &'a Table, key: &str) -> &'a str {
    table.get(key).and_then(Value::as_str).unwrap_or_default()
}

#[test]
fn the_starting_settings_hold_the_minted_key_as_nzbhydra2s_api_key_and_nothing_else_of_it() {
    let starting = read(STARTING);
    let placeholder = format!("{{{{secret:{KEY}}}}}");
    assert_eq!(starting.matches(&placeholder).count(), 1);
    assert!(starting
        .lines()
        .any(|line| line == format!("  apiKey: \"{placeholder}\"")));
    assert_eq!(starting.matches("{{").count(), 1);
}

#[test]
fn the_key_is_nzbhydra2s_first_minted_secret() {
    let manifest = manifest();
    let minted: Vec<&str> = listed(&manifest, "secret")
        .into_iter()
        .filter(|secret| text(secret, "of") == SERVICE && secret.contains_key("minted"))
        .map(|secret| text(secret, "id"))
        .collect();
    assert_eq!(minted.first(), Some(&KEY));
}

#[test]
fn no_adapter_is_handed_the_key() {
    let manifest = manifest();
    let handed: Vec<String> = listed(&manifest, "service")
        .into_iter()
        .filter_map(|service| service.get("upstream").and_then(Value::as_table))
        .flat_map(Table::values)
        .filter_map(|field| field.get("secret").and_then(Value::as_str))
        .map(str::to_owned)
        .collect();
    assert!(!handed.is_empty());
    assert!(!handed.iter().any(|secret| secret == KEY), "{handed:?}");
}
