use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

#[test]
pub(crate) fn member_short_name_strips_dir_and_git_suffix() {
    assert_eq!(member_short_name("repos/app.git"), "app");
    assert_eq!(member_short_name("app"), "app");
    assert_eq!(member_short_name("a/b/c"), "c");
    assert_eq!(member_short_name("x/"), "x");
}
