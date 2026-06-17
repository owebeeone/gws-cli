use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

#[test]
pub(crate) fn human_bytes_uses_binary_units() {
    assert_eq!(human_bytes(0), "0 B");
    assert_eq!(human_bytes(512), "512 B");
    assert_eq!(human_bytes(1024), "1.0 KiB");
    assert_eq!(human_bytes(1536), "1.5 KiB");
    assert_eq!(human_bytes(1_048_576), "1.0 MiB");
    assert_eq!(human_bytes(1_073_741_824), "1.0 GiB");
    assert_eq!(human_bytes(-5), "0 B");
}
