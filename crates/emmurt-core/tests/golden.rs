//! Snapshots of what the compiler writes for every `examples/*.emmurt` and
//! every `tests/golden/cases/*.emmurt`: the HTML in a `.html` file, or, for
//! `*.sbagliato.emmurt` cases, the error in a `.err` file.
//!
//! After an intended change: `EMMURT_UPDATE_GOLDEN=1 cargo test -p emmurt-core --test golden`.

use std::fs;
use std::path::{Path, PathBuf};

use emmurt_core::{compile, FratmSrc, Options};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `(source, snapshot)` pairs, paths relative to the repository.
fn cases() -> Vec<(String, String)> {
    let mut cases = Vec::new();
    for (dir, snapshots) in [
        ("examples", "crates/emmurt-core/tests/golden/examples"),
        (
            "crates/emmurt-core/tests/golden/cases",
            "crates/emmurt-core/tests/golden/cases",
        ),
    ] {
        let mut files: Vec<_> = fs::read_dir(repo().join(dir))
            .unwrap()
            .filter_map(|e| e.ok()?.file_name().into_string().ok())
            .filter(|n| n.ends_with(".emmurt"))
            .collect();
        files.sort();
        for name in files {
            let stem = name.trim_end_matches(".emmurt");
            let ext = if stem.ends_with(".sbagliato") {
                "err"
            } else {
                "html"
            };
            cases.push((format!("{dir}/{name}"), format!("{snapshots}/{stem}.{ext}")));
        }
    }
    cases
}

#[test]
fn golden() {
    let update = std::env::var_os("EMMURT_UPDATE_GOLDEN").is_some();
    // Paths starting with `/` are from the repository root
    let read = |path: &str| fs::read_to_string(repo().join(path.trim_start_matches('/'))).ok();
    let mut failures = Vec::new();
    for (source, snapshot) in cases() {
        let options = Options {
            filename: &source,
            read: &read,
            fratm_src: FratmSrc::Inline,
        };
        let text = fs::read_to_string(repo().join(&source)).unwrap();
        let actual = match (compile(&text, &options), snapshot.ends_with(".err")) {
            (Ok(out), false) => out.html,
            (Err(e), true) => format!("{e}\n"),
            (Ok(_), true) => {
                failures.push(format!("{source}: compiled, but it should fail"));
                continue;
            }
            (Err(e), false) => {
                failures.push(format!("{source}: {e}"));
                continue;
            }
        };
        let path = repo().join(&snapshot);
        if update {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, &actual).unwrap();
        } else if fs::read_to_string(&path).ok().as_deref() != Some(actual.as_str()) {
            failures.push(format!("{snapshot} is out of date:\n{actual}"));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n\n"));
}

#[test]
fn readme_lists_the_whole_vocabulary() {
    use emmurt_core::vocabolario::{ATTRIBUTES, EVENTS, TAGS};
    let readme = fs::read_to_string(repo().join("README.md")).unwrap();
    let words = TAGS
        .iter()
        .map(|(nap, _, _)| format!("`<{nap}>`"))
        .chain(ATTRIBUTES.iter().map(|a| format!("`{}`", a.nap)))
        .chain(EVENTS.iter().map(|(nap, _, _)| format!("`{nap}`")));
    let missing: Vec<String> = words.filter(|w| !readme.contains(w.as_str())).collect();
    assert!(missing.is_empty(), "README.md misses {missing:?}");
}
