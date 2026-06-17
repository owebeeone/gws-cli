use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

#[test]
pub(crate) fn usage_text_covers_standard_help_and_commands() {
    let usage = usage_text();

    assert!(usage.contains("Usage: gwz"));
    assert!(usage.contains("-h, --help"));
    assert!(usage.contains("init"));
    assert!(usage.contains("status"));
}
