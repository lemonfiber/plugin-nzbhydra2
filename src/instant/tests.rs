use std::time::{Duration, UNIX_EPOCH};

use super::{from_seconds, written};

#[test]
fn a_moment_is_written_as_the_utc_instant_nzbhydra2_reads() {
    assert_eq!(
        written(UNIX_EPOCH + Duration::from_secs(1_893_456_000)).as_deref(),
        Some("2030-01-01T00:00:00Z")
    );
    assert_eq!(
        written(UNIX_EPOCH + Duration::from_secs(1_792_890_558)).as_deref(),
        Some("2026-10-25T01:09:18Z")
    );
    assert_eq!(written(UNIX_EPOCH - Duration::from_secs(1)), None);
}

#[test]
fn seconds_nzbhydra2_writes_are_read_to_the_second() {
    assert_eq!(
        from_seconds(1_792_890_558.423),
        Some(UNIX_EPOCH + Duration::from_secs(1_792_890_558))
    );
    assert_eq!(from_seconds(-1.0), None);
    assert_eq!(from_seconds(f64::NAN), None);
}
