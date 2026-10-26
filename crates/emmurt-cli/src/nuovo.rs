//! `emmurt nuovo <folder>`: a ready Vite app with htmlemmurt and FratmScript.

use std::path::Path;

/// The template, as (path in the new app, content). `{{nomme}}` becomes the
/// app name and `{{versione}}` the version of htmlemmurt to install.
const FILES: &[(&str, &str)] = &[
    ("package.json", include_str!("nuovo/package.json")),
    ("vite.config.js", include_str!("nuovo/vite.config.js")),
    ("index.emmurt", include_str!("nuovo/index.emmurt")),
    (
        "pezzi/cappiell.emmurt",
        include_str!("nuovo/pezzi/cappiell.emmurt"),
    ),
    ("pezzi/pere.emmurt", include_str!("nuovo/pezzi/pere.emmurt")),
    ("src/main.fratm", include_str!("nuovo/src/main.fratm")),
    ("stile.css", include_str!("nuovo/stile.css")),
    // Named without the dot here, so git does not read it as this repo's ignore file
    (".gitignore", include_str!("nuovo/gitignore")),
];

/// Writes the app into `dir`, which must not exist or be empty. Returns the
/// app name.
pub fn create(dir: &Path) -> Result<String, String> {
    if dir.read_dir().is_ok_and(|mut d| d.next().is_some()) {
        return Err(format!("'{}' esiste già e nun è vacante.", dir.display()));
    }
    let name = app_name(dir);
    for (path, content) in FILES {
        let text = content
            .replace("{{nomme}}", &name)
            .replace("{{versione}}", env!("CARGO_PKG_VERSION"));
        crate::write_file(&dir.join(path), &text)?;
    }
    Ok(name)
}

/// A valid npm package name from the folder name.
fn app_name(dir: &Path) -> String {
    let base = dir
        .canonicalize()
        .unwrap_or_else(|_| dir.to_path_buf())
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let name: String = base
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let name = name.trim_matches('-');
    if name.is_empty() {
        "paggena".to_string()
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use emmurt_core::{compile, FratmSrc, Options};
    use std::fs;

    fn temp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("emmurt-nuovo-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn creates_an_app_that_compiles() {
        let dir = temp("A Pizzeria!");
        assert_eq!(
            create(&dir).unwrap(),
            "emmurt-nuovo-".to_string() + &format!("{}-a-pizzeria", std::process::id())
        );
        let package = fs::read_to_string(dir.join("package.json")).unwrap();
        assert!(package.contains(&format!(
            "\"htmlemmurt\": \"^{}\"",
            env!("CARGO_PKG_VERSION")
        )));
        assert!(!package.contains("{{"));
        let root = dir.clone();
        // As the CLI reads: the path as it is, or `/…` from the app folder
        let read = move |p: &str| {
            fs::read_to_string(p)
                .or_else(|_| fs::read_to_string(root.join(p.trim_start_matches('/'))))
                .ok()
        };
        // Inlined, `<fratm da="/src/main.fratm" />` compiles the module too
        for page in ["index.emmurt", "pezzi/cappiell.emmurt", "pezzi/pere.emmurt"] {
            let source = fs::read_to_string(dir.join(page)).unwrap();
            let filename = dir.join(page).to_string_lossy().to_string();
            let options = Options {
                filename: &filename,
                read: &read,
                fratm_src: FratmSrc::Inline,
            };
            compile(&source, &options).unwrap_or_else(|e| panic!("{e}"));
        }
        assert!(dir.join(".gitignore").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn refuses_a_folder_with_files() {
        let dir = temp("piena");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("x"), "").unwrap();
        assert!(create(&dir).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }
}
