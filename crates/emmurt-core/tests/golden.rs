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

/// Every `html` block of the guide compiles and is formatted. A block with a
/// `<chiamm>` reads the block before it as the piece it calls.
#[test]
fn guide_examples_compile() {
    let guide = fs::read_to_string(repo().join("docs/guida.md")).unwrap();
    let blocks: Vec<&str> = guide
        .split("```html\n")
        .skip(1)
        .map(|b| b.split("```").next().unwrap())
        .collect();
    assert!(blocks.len() >= 5);
    for (i, block) in blocks.iter().enumerate() {
        let piece = i.checked_sub(1).map(|p| blocks[p].to_string());
        let read = |_: &str| piece.clone();
        let options = Options {
            read: &read,
            ..Options::default()
        };
        if let Err(e) = compile(block, &options) {
            panic!("docs/guida.md block {}: {e}\n{block}", i + 1);
        }
        assert_eq!(emmurt_core::format(block, "guida").unwrap(), *block);
    }
}

/// The playground's guide and lessons: every snippet compiles (the
/// `*.sbagliato.emmurt` ones must fail) and is formatted, and every lesson's
/// solution is a different page from its start. `<chiamm>` reads
/// `examples/pezzi`, as in the playground.
#[test]
fn playground_snippets_compile() {
    let examples = repo().join("examples");
    let read = |path: &str| fs::read_to_string(examples.join(path)).ok();
    let options = Options {
        read: &read,
        ..Options::default()
    };
    for dir in ["guida", "scola"] {
        let dir = repo().join("packages/playground/src").join(dir);
        let mut files: Vec<PathBuf> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "emmurt"))
            .collect();
        files.sort();
        assert!(files.len() >= 9, "{}", dir.display());
        for path in files {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let source = fs::read_to_string(&path).unwrap();
            let html = compile(&source, &options).map(|out| out.html);
            assert_eq!(
                html.is_err(),
                name.ends_with(".sbagliato.emmurt"),
                "{name}: {:?}",
                html.as_ref().err()
            );
            assert_eq!(
                emmurt_core::format(&source, &name).unwrap(),
                source,
                "{name} is not formatted"
            );
            if let Some(start) = name.strip_suffix(".soluzione.emmurt") {
                let starter = fs::read_to_string(dir.join(format!("{start}.emmurt"))).unwrap();
                let starter = compile(&starter, &options).unwrap().html;
                assert_ne!(
                    starter,
                    html.unwrap(),
                    "{name} is the same page as its start"
                );
            }
        }
    }
}
