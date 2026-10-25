//! The `emmurt` command: builds, checks and watches htmlemmurt pages.

use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, SystemTime};

use emmurt_core::{compile, vocabolario, Error, FratmSrc, Options};
use serde_json::json;

const AIUTO: &str = "\
emmurt: HTML, ma comme si deve

  emmurt build <file|cartella...>   Compila 'e pagine .emmurt in .html
      --out <cartella>              Scrive llà invece che vicino ê sorgenti
      --stdout                      Stampa l'HTML (nu file sulo, o - pe' stdin)
  emmurt check <file|cartella...>   Controlla senza scrivere niente
  emmurt watch <file|cartella...>   Ricompila ogni vota ca cagne quaccosa
  emmurt vocabolario                'E parole napoletane e l'HTML lloro

  --json   Na risposta JSON sola, pe' editor e agenti

Na cartella compila sulo 'e pagine (file ca accummenciano cu <cristian>):
'e pezzi pe' <chiamm> nun diventano .html.
";

#[derive(Default)]
struct Flags {
    json: bool,
    stdout: bool,
    out: Option<PathBuf>,
}

/// A file to compile, and the folder named on the command line it came from.
struct Input {
    path: PathBuf,
    root: PathBuf,
    from_folder: bool,
}

/// What compiling one input gave.
struct Compiled {
    input: Input,
    result: Result<(String, Vec<String>), Error>,
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_default();
    let mut flags = Flags::default();
    let mut paths = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => flags.json = true,
            "--stdout" => flags.stdout = true,
            "--out" => match args.next() {
                Some(dir) => flags.out = Some(dir.into()),
                None => return fail("--out vo' na cartella."),
            },
            s if s.starts_with("--") => return fail(&format!("'{s}' nun 'o cunosco.")),
            _ => paths.push(arg),
        }
    }
    match command.as_str() {
        "build" | "check" | "watch" if paths.is_empty() => {
            fail("Dimme quale file: emmurt build index.emmurt")
        }
        "build" => build(&paths, &flags, true),
        "check" => build(&paths, &flags, false),
        "watch" => watch(&paths, &flags),
        "vocabolario" => vocabulary(flags.json),
        "--version" | "-V" => {
            println!("emmurt {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        "" | "aiuto" | "--help" | "-h" => {
            print!("{AIUTO}");
            ExitCode::SUCCESS
        }
        other => fail(&format!("'{other}' nun è nu comando. Prova: emmurt aiuto")),
    }
}

fn fail(message: &str) -> ExitCode {
    eprintln!("✗ {message}");
    ExitCode::FAILURE
}

fn build(paths: &[String], flags: &Flags, write: bool) -> ExitCode {
    if paths == ["-"] {
        return build_stdin(flags, write);
    }
    let inputs = match inputs(paths, flags.out.as_deref()) {
        Ok(inputs) => inputs,
        Err(e) => return fail(&e),
    };
    if flags.stdout && inputs.len() != 1 {
        return fail("--stdout vo' nu file sulo.");
    }
    let compiled: Vec<Compiled> = inputs.into_iter().map(compile_input).collect();
    let mut errors = Vec::new();
    let mut written = Vec::new();
    for c in &compiled {
        match &c.result {
            Err(e) => errors.push(e.clone()),
            Ok((html, _)) if flags.stdout => print!("{html}"),
            Ok((html, _)) if write => {
                // A folder builds pages only: fragments are pasted in by <chiamm>
                if c.input.from_folder && !html.starts_with("<!DOCTYPE html>") {
                    continue;
                }
                let out = output_path(&c.input, flags.out.as_deref());
                if let Err(e) = out
                    .parent()
                    .map_or(Ok(()), fs::create_dir_all)
                    .and_then(|_| fs::write(&out, html))
                {
                    return fail(&format!("Nun riesco a scrivere '{}': {e}", out.display()));
                }
                written.push(out.display().to_string());
            }
            Ok(_) => {}
        }
    }
    report(&errors, flags.json, json!({ "written": written }), None);
    if !flags.json && !flags.stdout {
        for w in &written {
            eprintln!("✓ {w}");
        }
        if !write && errors.is_empty() {
            eprintln!("✓ Tutto a posto: {} file.", compiled.len());
        }
    }
    exit(&errors)
}

fn build_stdin(flags: &Flags, write: bool) -> ExitCode {
    let mut source = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut source) {
        return fail(&format!("Nun riesco a leggere stdin: {e}"));
    }
    let read = |path: &str| read_file(path, Path::new("."));
    let options = Options {
        filename: "stdin.emmurt",
        read: &read,
        fratm_src: FratmSrc::Inline,
    };
    match compile(&source, &options) {
        Ok(out) if flags.json => {
            report(
                &[],
                true,
                json!({ "html": write.then_some(&out.html) }),
                None,
            );
            ExitCode::SUCCESS
        }
        Ok(out) => {
            if write {
                print!("{}", out.html);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            report(
                std::slice::from_ref(&e),
                flags.json,
                json!({}),
                Some(&source),
            );
            ExitCode::FAILURE
        }
    }
}

fn watch(paths: &[String], flags: &Flags) -> ExitCode {
    loop {
        let inputs = match inputs(paths, flags.out.as_deref()) {
            Ok(inputs) => inputs,
            Err(e) => return fail(&e),
        };
        let mut watched: Vec<PathBuf> = inputs.iter().map(|i| i.path.clone()).collect();
        for c in inputs.into_iter().map(compile_input) {
            match &c.result {
                Ok((html, deps)) => {
                    watched.extend(deps.iter().map(|d| resolve(d, &c.input.root)));
                    if c.input.from_folder && !html.starts_with("<!DOCTYPE html>") {
                        continue;
                    }
                    let out = output_path(&c.input, flags.out.as_deref());
                    let written = out
                        .parent()
                        .map_or(Ok(()), fs::create_dir_all)
                        .and_then(|_| fs::write(&out, html));
                    match written {
                        Ok(()) => eprintln!("✓ {} → {}", c.input.path.display(), out.display()),
                        Err(e) => eprintln!("✗ Nun riesco a scrivere '{}': {e}", out.display()),
                    }
                }
                Err(e) => print_error(e, None),
            }
        }
        eprintln!("… aspetto ca cagne quaccosa (Ctrl+C pe' fernì)");
        let seen: Vec<(PathBuf, Option<SystemTime>)> = watched
            .into_iter()
            .map(|p| (p.clone(), modified(&p)))
            .collect();
        // ponytail: polls every 300 ms; a file watcher crate if this gets slow
        loop {
            std::thread::sleep(Duration::from_millis(300));
            let folders_changed = paths.iter().any(|p| {
                Path::new(p).is_dir()
                    && collect(Path::new(p), flags.out.as_deref())
                        .iter()
                        .any(|f| !seen.iter().any(|(s, _)| s == f))
            });
            if folders_changed || seen.iter().any(|(p, t)| modified(p) != *t) {
                break;
            }
        }
    }
}

fn modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

fn vocabulary(as_json: bool) -> ExitCode {
    if as_json {
        let out = json!({
            "tags": vocabolario::TAGS,
            "attributes": vocabolario::ATTRS,
            "events": vocabolario::EVENTS,
        });
        println!("{out}");
        return ExitCode::SUCCESS;
    }
    println!("Elementi");
    for (nap, html) in vocabolario::TAGS {
        println!("  <{nap}>  →  <{html}>");
    }
    println!("  <chiamm da=\"…\" />  →  'o file, ncullato llà");
    println!("\nAttribute");
    for a in vocabolario::ATTRS {
        let only = if a.only.is_empty() {
            String::new()
        } else {
            format!("  (ncopp'a <{}>)", a.only.join(">, <"))
        };
        println!("  {}  →  {}{only}", a.nap, a.html);
        for (nap, html) in a.values {
            println!("      \"{nap}\"  →  \"{html}\"");
        }
    }
    println!("\nEventi (dint'ê virgulette ce va FratmScript)");
    for (nap, html) in vocabolario::EVENTS {
        println!("  {nap}  →  {html}");
    }
    ExitCode::SUCCESS
}

/// Files named on the command line, folders expanded to their `.emmurt` files.
fn inputs(paths: &[String], out: Option<&Path>) -> Result<Vec<Input>, String> {
    let mut inputs = Vec::new();
    for p in paths {
        let path = Path::new(p);
        if path.is_dir() {
            inputs.extend(collect(path, out).into_iter().map(|f| Input {
                path: f,
                root: path.to_path_buf(),
                from_folder: true,
            }));
        } else if path.is_file() {
            inputs.push(Input {
                path: path.to_path_buf(),
                root: path.parent().unwrap_or(Path::new("")).to_path_buf(),
                from_folder: false,
            });
        } else {
            return Err(format!("Nun trovo '{p}'."));
        }
    }
    Ok(inputs)
}

/// The `.emmurt` files under a folder, skipping hidden folders,
/// `node_modules` and the output folder.
fn collect(dir: &Path, out: Option<&Path>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return files;
    };
    let mut entries: Vec<PathBuf> = entries.filter_map(|e| Some(e.ok()?.path())).collect();
    entries.sort();
    for path in entries {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if path.is_dir() {
            let skip = name.starts_with('.')
                || name == "node_modules"
                || out.is_some_and(|o| same_path(o, &path));
            if !skip {
                files.extend(collect(&path, out));
            }
        } else if name.ends_with(".emmurt") {
            files.push(path);
        }
    }
    files
}

fn same_path(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

fn compile_input(input: Input) -> Compiled {
    let filename = input.path.to_string_lossy().replace('\\', "/");
    let result = match fs::read_to_string(&input.path) {
        Ok(source) => {
            let root = input.root.clone();
            let read = move |path: &str| read_file(path, &root);
            let options = Options {
                filename: &filename,
                read: &read,
                fratm_src: FratmSrc::Inline,
            };
            compile(&source, &options).map(|out| (out.html, out.dependencies))
        }
        Err(e) => Err(Error {
            message: format!("Nun riesco a leggere stu file: {e}"),
            file: filename.clone(),
            line: 1,
            column: 1,
            suggestion: None,
        }),
    };
    Compiled { input, result }
}

/// A path from the compiler: as written, or, when it starts with `/`,
/// under the folder named on the command line.
fn resolve(path: &str, root: &Path) -> PathBuf {
    let direct = PathBuf::from(path);
    match path.strip_prefix('/') {
        Some(rest) if !direct.exists() => root.join(rest),
        _ => direct,
    }
}

fn read_file(path: &str, root: &Path) -> Option<String> {
    fs::read_to_string(resolve(path, root)).ok()
}

fn output_path(input: &Input, out: Option<&Path>) -> PathBuf {
    let relative = match out {
        Some(dir) => dir.join(input.path.strip_prefix(&input.root).unwrap_or(&input.path)),
        None => input.path.clone(),
    };
    relative.with_extension("html")
}

fn report(errors: &[Error], as_json: bool, extra: serde_json::Value, stdin: Option<&str>) {
    if as_json {
        let mut out = json!({ "ok": errors.is_empty(), "errors": errors });
        if let (Some(out), Some(extra)) = (out.as_object_mut(), extra.as_object()) {
            out.extend(extra.clone());
        }
        println!("{out}");
    } else {
        for e in errors {
            print_error(e, stdin);
        }
    }
}

fn exit(errors: &[Error]) -> ExitCode {
    if errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// The error, the line it is on with a caret under the column, and the hint.
fn print_error(e: &Error, stdin: Option<&str>) {
    eprintln!("✗ {}:{}:{}: {}", e.file, e.line, e.column, e.message);
    let source = match stdin {
        Some(s) if e.file == "stdin.emmurt" => Some(s.to_string()),
        _ => fs::read_to_string(&e.file).ok(),
    };
    if let Some(line) = source
        .as_deref()
        .and_then(|s| s.lines().nth(e.line.saturating_sub(1)))
    {
        let pad: String = line
            .chars()
            .take(e.column.saturating_sub(1))
            .map(|c| if c == '\t' { '\t' } else { ' ' })
            .collect();
        eprintln!("  {:>4} | {line}", e.line);
        eprintln!("       | {pad}^");
    }
    if let Some(s) = &e.suggestion {
        eprintln!("  {s}");
    }
}
