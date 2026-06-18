use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn package_name_uses_project_token() {
    assert_eq!(env!("CARGO_PKG_NAME"), "gwz");
}

#[test]
fn repository_sources_use_project_token_everywhere() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let stale_tokens = stale_tokens();
    let mut stale_hits = Vec::new();

    collect_stale_hits(&root, &root, &stale_tokens, &mut stale_hits);

    assert!(
        stale_hits.is_empty(),
        "stale project spelling remains:\n{}",
        stale_hits.join("\n")
    );
}

fn stale_tokens() -> Vec<String> {
    vec![
        String::from_utf8(vec![103, 119, 115]).unwrap(),
        String::from_utf8(vec![71, 87, 83]).unwrap(),
        String::from_utf8(vec![71, 119, 115]).unwrap(),
    ]
}

fn collect_stale_hits(
    root: &Path,
    dir: &Path,
    stale_tokens: &[String],
    stale_hits: &mut Vec<String>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if should_skip(&path) {
            continue;
        }
        let relative = path.strip_prefix(root).unwrap_or(path.as_path());
        let relative_text = relative.to_string_lossy();
        let relative_display = relative_text.to_string();
        for token in stale_tokens {
            if relative_text.contains(token) {
                stale_hits.push(relative_display.clone());
            }
        }
        if path.is_dir() {
            collect_stale_hits(root, &path, stale_tokens, stale_hits);
        } else if let Ok(contents) = fs::read_to_string(&path) {
            for token in stale_tokens {
                if contents.contains(token) {
                    stale_hits.push(relative_display.clone());
                    break;
                }
            }
        }
    }
}

fn should_skip(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == ".git" || name == "target")
}
