//! # htmlemmurt
//!
//! HTML in Neapolitan. The compiler reads markup with Neapolitan element,
//! attribute and value names and writes standard HTML:
//!
//! ```rust
//! use emmurt_core::{compile, Options};
//!
//! let out = compile("<cascett classe=\"menu\">Uè!</cascett>", &Options::default()).unwrap();
//! assert_eq!(out.html, "<div class=\"menu\">Uè!</div>");
//! ```
//!
//! It is strict: English names, tags closed in the wrong order and unclosed
//! tags are errors, in Neapolitan, with line and column. The words are the
//! ones of `fratm_core::html`, shared with FratmScript's JSX. `<fratm>` blocks
//! and `quanno-*` handlers are FratmScript, compiled to JavaScript with
//! `fratm-core`; `<cess>` and `cess="…"` are CSS in Neapolitan, translated by
//! its `stile`. `<chiamm da="pezzi/menu.emmurt" />` pastes another file in.

mod format;
mod lexer;
pub mod vocabolario;

use std::fmt;

pub use format::format;

use lexer::{Pos, Tag, Token};
use serde::Serialize;
use vocabolario::{
    Attribute, INCLUDE, UNTRANSLATED_ATTRIBUTES, UNTRANSLATED_EVENTS, UNTRANSLATED_TAGS, VOID,
};

/// What to do with `<fratm da="file.fratm">`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FratmSrc {
    /// Keep it as `<script type="module" src="file.fratm">`: a bundler (the
    /// Vite plugin) compiles the file and the modules it imports.
    #[default]
    Keep,
    /// Read the file, compile it and write its JavaScript into the page. No
    /// bundler needed, but it cannot import other `.fratm` files.
    Inline,
}

pub struct Options<'a> {
    /// Name of the page, used in errors and to resolve relative paths.
    pub filename: &'a str,
    /// Reads a file named by `<chiamm da>` or an inlined `<fratm da>`, or
    /// None when it cannot. Gets the path joined to the directory of the file
    /// that names it, or as written when it starts with `/`.
    pub read: &'a dyn Fn(&str) -> Option<String>,
    pub fratm_src: FratmSrc,
}

impl Default for Options<'_> {
    fn default() -> Self {
        Options {
            filename: "input.emmurt",
            read: &|_| None,
            fratm_src: FratmSrc::Keep,
        }
    }
}

#[derive(Debug)]
pub struct Output {
    pub html: String,
    /// Files read while compiling (includes and inlined scripts).
    pub dependencies: Vec<String>,
}

/// A compile error. The message is in Neapolitan; `suggestion` says how to fix it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Error {
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub suggestion: Option<String>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}",
            self.file, self.line, self.column, self.message
        )?;
        if let Some(s) = &self.suggestion {
            write!(f, "\n  {s}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

/// An error before we know which file it is in.
#[derive(Debug)]
pub(crate) struct Fail {
    pub pos: Pos,
    pub message: String,
    pub suggestion: Option<String>,
}

impl Fail {
    pub(crate) fn new(pos: Pos, message: impl Into<String>) -> Self {
        Fail {
            pos,
            message: message.into(),
            suggestion: None,
        }
    }

    pub(crate) fn hint(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    fn in_file(self, file: &str) -> Error {
        Error {
            message: self.message,
            file: file.to_string(),
            line: self.pos.line,
            column: self.pos.column,
            suggestion: self.suggestion,
        }
    }
}

/// Compiles a page or a fragment. A page is a file whose first element is
/// `<cristian>`: it also gets `<!DOCTYPE html>` and, when it has none, a
/// `<meta charset="utf-8">`.
pub fn compile(source: &str, options: &Options) -> Result<Output, Error> {
    let mut compiler = Compiler {
        options,
        chain: vec![options.filename.replace('\\', "/")],
        dependencies: Vec::new(),
        pieces: Vec::new(),
    };
    let html = compiler.fragment(source)?;
    let html = if is_page(&html) { page(html) } else { html };
    Ok(Output {
        html,
        dependencies: compiler.dependencies,
    })
}

struct Compiler<'a> {
    options: &'a Options<'a>,
    /// The page and the includes being compiled, innermost last.
    chain: Vec<String>,
    dependencies: Vec<String>,
    /// The parameters of the pieces being compiled, innermost last.
    pieces: Vec<Piece>,
}

/// The placeholder that gets what stands between `<chiamm>` and `</chiamm>`.
const CONTENT: &str = "cuntenuto";

/// A piece called with parameters, while it is compiled.
struct Piece {
    /// The path as written in `da`.
    path: String,
    /// (name, value, position of the attribute in the caller).
    values: Vec<(String, String, Pos)>,
    /// The compiled HTML between the tags, when they are not self-closing.
    content: Option<String>,
    used: Vec<String>,
    /// The file and position of the `<chiamm>`, where a missing value is fixed.
    caller: String,
    at: Pos,
}

impl Piece {
    fn error(&self, message: String, hint: String) -> Error {
        Error {
            message,
            file: self.caller.clone(),
            line: self.at.line,
            column: self.at.column,
            suggestion: Some(hint),
        }
    }
}

/// An open element: the name as written and where it opened.
struct Open {
    name: String,
    pos: Pos,
}

/// Closes the innermost open element with `</name>`, or says why it cannot.
fn close(open: &mut Vec<Open>, name: &str, html: &str, pos: Pos) -> Result<(), Fail> {
    if VOID.contains(&html) {
        return Err(
            Fail::new(pos, format!("<{name}> nun se chiude: è sempe vacante."))
                .hint(format!("Leva </{name}>.")),
        );
    }
    let same = |o: &Open| o.name.to_lowercase() == name.to_lowercase();
    match open.last() {
        Some(last) if same(last) => {
            open.pop();
            Ok(())
        }
        Some(last) if open.iter().any(same) => Err(Fail::new(
            pos,
            format!(
                "Hê chiuso </{name}> ma <{}> (riga {}) è ancora araputo.",
                last.name, last.pos.line
            ),
        )
        .hint(format!("Chiudi primma </{}>.", last.name))),
        last => {
            let fail = Fail::new(
                pos,
                format!("</{name}> chiude na cosa ca nun è maje stata araputa."),
            );
            Err(match last {
                Some(l) => fail.hint(format!(
                    "L'urdemo araputo è <{}> (riga {}).",
                    l.name, l.pos.line
                )),
                None => fail.hint(format!("Leva </{name}>.")),
            })
        }
    }
}

/// The error for an element still open at the end of the file.
fn unclosed(open: &[Open]) -> Result<(), Fail> {
    match open.last() {
        Some(o) => Err(
            Fail::new(o.pos, format!("<{}> nun l'hê maje chiuso.", o.name))
                .hint(format!("Scrive </{}> quanno fernesce.", o.name)),
        ),
        None => Ok(()),
    }
}

impl Compiler<'_> {
    fn file(&self) -> &str {
        self.chain.last().map(String::as_str).unwrap_or_default()
    }

    fn fail(&self, fail: Fail) -> Error {
        fail.in_file(self.file())
    }

    fn fragment(&mut self, source: &str) -> Result<String, Error> {
        let tokens = lexer::tokenize(source).map_err(|f| self.fail(f))?;
        self.tokens(tokens)
    }

    fn tokens(&mut self, tokens: Vec<Token>) -> Result<String, Error> {
        let mut tokens = tokens.into_iter();
        let mut out = String::new();
        let mut open: Vec<Open> = Vec::new();
        while let Some(token) = tokens.next() {
            match token {
                Token::Text(text) => out.push_str(&self.fill(&text, true)?),
                Token::Raw { text, .. } => out.push_str(&text),
                Token::Start(tag) if tag.name.to_lowercase() == INCLUDE => {
                    let tag = self.fill_tag(tag)?;
                    let content = match tag.self_closing {
                        true => None,
                        false => Some(self.content(&tag, &mut tokens)?),
                    };
                    out.push_str(&self.include(&tag, content)?);
                }
                Token::Start(tag) => {
                    let tag = self.fill_tag(tag)?;
                    let (html, custom) = self.element(&tag.name, tag.pos)?;
                    let mut attrs = self.attrs(&tag, &html, custom)?;
                    // The lexer puts the content right after a start tag that is not self-closing
                    let mut raw = || match tag.self_closing {
                        true => None,
                        false => match tokens.next() {
                            Some(Token::Raw { text, pos }) => Some((text, pos)),
                            _ => None,
                        },
                    };
                    let content = match html.as_str() {
                        "script" => self.script(&tag, &mut attrs, raw())?,
                        "style" => match raw() {
                            Some((text, pos)) => self.css(&text, pos)?,
                            None => String::new(),
                        },
                        _ => String::new(),
                    };
                    out.push('<');
                    out.push_str(&html);
                    for (name, value) in &attrs {
                        out.push(' ');
                        out.push_str(name);
                        if let Some(v) = value {
                            out.push_str("=\"");
                            out.push_str(v);
                            out.push('"');
                        }
                    }
                    out.push('>');
                    out.push_str(&content);
                    if VOID.contains(&html.as_str()) {
                        continue;
                    }
                    if tag.self_closing {
                        out.push_str(&format!("</{html}>"));
                    } else {
                        open.push(Open {
                            name: tag.name,
                            pos: tag.pos,
                        });
                    }
                }
                Token::End { name, pos } => {
                    let (html, _) = self.element(&name, pos)?;
                    close(&mut open, &name, &html, pos).map_err(|f| self.fail(f))?;
                    out.push_str(&format!("</{html}>"));
                }
            }
        }
        unclosed(&open).map_err(|f| self.fail(f))?;
        Ok(out)
    }

    /// The HTML name of an element, and whether it is custom (attributes it
    /// does not know pass through as written).
    fn element(&self, name: &str, pos: Pos) -> Result<(String, bool), Error> {
        let lower = name.to_lowercase();
        if lower.contains('-') {
            return Ok((name.to_string(), true));
        }
        if let Some(html) = vocabolario::tag(&lower) {
            return Ok((html.to_string(), html == "svg"));
        }
        // No Neapolitan word yet: the HTML name as it is
        if UNTRANSLATED_TAGS.contains(&lower.as_str()) {
            return Ok((lower, false));
        }
        let fail = if let Some(nap) = vocabolario::tag_for_html(&lower) {
            let hint = if lower == "script" {
                "Vuoi dire <fratm>? Pe' JavaScript normale miette tipo=\"text/javascript\"."
                    .to_string()
            } else {
                format!("Vuoi dire <{nap}>?")
            };
            Fail::new(pos, format!("<{name}> nun è napoletano.")).hint(hint)
        } else {
            let names = vocabolario::TAGS.iter().map(|(n, _, _)| *n);
            let hint = match vocabolario::closest(&lower, names) {
                Some(n) => format!("Vuoi dire <{n}>?"),
                None => "'E nomme stanno int'a tabella: emmurt vocabolario. \
                         N'elemento tuojo vo' nu trattino: <pizza-margherita>."
                    .to_string(),
            };
            Fail::new(pos, format!("<{name}> nun 'o cunosco.")).hint(hint)
        };
        Err(self.fail(fail))
    }

    fn attrs(
        &self,
        tag: &Tag,
        element: &str,
        custom: bool,
    ) -> Result<Vec<(String, Option<String>)>, Error> {
        let mut out: Vec<(String, Option<String>)> = Vec::new();
        for a in &tag.attrs {
            let lower = a.name.to_lowercase();
            let fail = |message: String| Fail::new(a.pos, message);
            let (name, value) = if lower.starts_with("data-") || lower.starts_with("aria-") {
                (a.name.clone(), a.value.as_deref().map(quote))
            } else if lower.starts_with("quanno-") {
                let Some(html) = vocabolario::event(&lower) else {
                    let names = vocabolario::EVENTS.iter().map(|(n, _, _)| *n);
                    let hint = match vocabolario::closest(&lower, names) {
                        Some(n) => format!("Vuoi dire '{n}'?"),
                        None => format!("'E eventi so': {}.", event_list()),
                    };
                    return Err(self.fail(fail(format!("'{}' nun 'o cunosco.", a.name)).hint(hint)));
                };
                let code = decode(a.value.as_deref().unwrap_or_default());
                let js = fratm(&code, self.file(), a.value_pos)?.code;
                (html.to_string(), Some(escape_attr(js.trim_end())))
            } else if let Some(attr) = vocabolario::attr_for(&lower, element) {
                let value = match &a.value {
                    Some(v) if attr.html == "style" => Some(quote(&self.css(v, a.value_pos)?)),
                    Some(v) => Some(self.value(attr, a, v)?),
                    None => None,
                };
                (attr.html.to_string(), value)
            } else if let Some(nap) = vocabolario::event_for_html(&lower) {
                return Err(self.fail(
                    fail(format!("'{}' nun è napoletano.", a.name))
                        .hint(format!("Vuoi dire '{nap}'?")),
                ));
            } else if let Some(attr) = vocabolario::attr_for_html(&lower, element) {
                return Err(self.fail(
                    fail(format!("'{}' nun è napoletano.", a.name))
                        .hint(format!("Vuoi dire '{}'?", attr.nap)),
                ));
            } else if UNTRANSLATED_EVENTS.contains(&lower.as_str())
                || UNTRANSLATED_ATTRIBUTES.contains(&lower.as_str())
            {
                // No Neapolitan word yet: the HTML name and value as written
                (lower.clone(), a.value.as_deref().map(quote))
            } else if lower.starts_with("on") && lower.len() > 2 {
                return Err(self.fail(
                    fail(format!("'{}' nun è napoletano.", a.name))
                        .hint(format!("'E eventi so': {}.", event_list())),
                ));
            } else if custom {
                (a.name.clone(), a.value.as_deref().map(quote))
            } else {
                let names = vocabolario::ATTRIBUTES
                    .iter()
                    .filter(|x| vocabolario::applies(x, element))
                    .map(|x| x.nap);
                let hint = match vocabolario::closest(&lower, names) {
                    Some(n) => format!("Vuoi dire '{n}'?"),
                    None => format!("Pe' n'attributo tuojo usa data-{lower}."),
                };
                return Err(self.fail(
                    fail(format!(
                        "<{}> nun cunosce l'attributo '{}'.",
                        tag.name, a.name
                    ))
                    .hint(hint),
                ));
            };
            if out.iter().any(|(n, _)| n.eq_ignore_ascii_case(&name)) {
                return Err(self.fail(fail(format!("'{}' l'hê scritto doje vote.", a.name))));
            }
            out.push((name, value));
        }
        Ok(out)
    }

    fn value(&self, attr: &Attribute, a: &lexer::Attr, value: &str) -> Result<String, Error> {
        let lower = value.trim().to_lowercase();
        if let Some((_, html)) = attr.values.iter().find(|(n, _)| *n == lower) {
            return Ok(html.to_string());
        }
        if let Some((nap, _)) = attr.values.iter().find(|(_, h)| *h == lower) {
            return Err(self.fail(
                Fail::new(
                    a.value_pos,
                    format!("{}=\"{value}\" nun è napoletano.", a.name),
                )
                .hint(format!("Vuoi dire {}=\"{nap}\"?", a.name)),
            ));
        }
        Ok(quote(value))
    }

    /// The content of a `<fratm>` element; may rewrite its attributes. It is
    /// FratmScript, unless `tipo` names something else (`application/ld+json`,
    /// `importmap`, `text/javascript`): then it is copied as written, as HTML
    /// does with `type`.
    fn script(
        &mut self,
        tag: &Tag,
        attrs: &mut Vec<(String, Option<String>)>,
        raw: Option<(String, Pos)>,
    ) -> Result<String, Error> {
        let kind = attrs.iter().find(|(n, _)| n == "type");
        if kind.is_some_and(|(_, v)| v.as_deref() != Some("module")) {
            return Ok(raw.map(|(text, _)| text).unwrap_or_default());
        }
        let has_code = raw.as_ref().is_some_and(|(t, _)| !t.trim().is_empty());
        let src = attrs.iter().position(|(n, _)| n == "src");
        let js = match src {
            Some(i) if has_code => {
                let pos = tag.attrs[i].pos;
                return Err(self.fail(
                    Fail::new(pos, "<fratm da=\"…\"> adda essere vacante.")
                        .hint("O scrive 'o codice ccà dinto, o 'o piglie da 'o file."),
                ));
            }
            Some(i) => {
                let src = attrs[i].1.clone().unwrap_or_default();
                // Only .fratm files are FratmScript: any other script loads as it is
                let path = src.split(['?', '#']).next().unwrap_or_default();
                if !path.ends_with(".fratm") {
                    return Ok(String::new());
                }
                let local = !src.contains("://") && !src.starts_with("//");
                if self.options.fratm_src == FratmSrc::Inline && local {
                    let path = join(self.file(), &src);
                    let source = self.read(&path, &src, tag.attrs[i].value_pos)?;
                    attrs.remove(i);
                    fratm(&source, &path, Pos { line: 1, column: 1 })?.code
                } else {
                    // A bundler compiles the file: it has to load as a module
                    if kind.is_none() {
                        attrs.push(("type".into(), Some("module".into())));
                    }
                    return Ok(String::new());
                }
            }
            None => match raw {
                Some((text, pos)) if has_code => fratm(&text, self.file(), pos)?.code,
                _ => String::new(),
            },
        };
        if is_module(&js) && !attrs.iter().any(|(n, _)| n == "type") {
            attrs.push(("type".into(), Some("module".into())));
        }
        Ok(format!("\n{}", escape_script(&js)))
    }

    /// CSS in Neapolitan that starts at `at`, translated by FratmScript's
    /// `stile`; plain CSS passes through.
    fn css(&self, text: &str, at: Pos) -> Result<String, Error> {
        match fratm_core::stile::translate(&[text.to_string()]) {
            Ok(mut chunks) => Ok(chunks.remove(0)),
            Err(e) => {
                let (line, column) = (e.position.line, e.position.column);
                let pos = if line == 0 {
                    Pos {
                        line: at.line,
                        column: at.column + column,
                    }
                } else {
                    Pos {
                        line: at.line + line,
                        column: column + 1,
                    }
                };
                Err(self.fail(Fail::new(pos, e.message)))
            }
        }
    }

    /// `<chiamm da="…" />`: the compiled content of another file. Its other
    /// attributes are parameters for the `{{…}}` placeholders of the file,
    /// and what stands between `<chiamm>` and `</chiamm>` goes in `{{cuntenuto}}`.
    fn include(&mut self, tag: &Tag, content: Option<String>) -> Result<String, Error> {
        let Some((da, pos)) = tag
            .attrs
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case("da"))
            .and_then(|a| Some((a.value.clone()?, a.value_pos)))
        else {
            return Err(self.fail(
                Fail::new(tag.pos, "<chiamm> vo' sapé addò sta 'o file.")
                    .hint("Scrive <chiamm da=\"pezzi/menu.emmurt\" />."),
            ));
        };
        let mut values: Vec<(String, String, Pos)> = Vec::new();
        for a in tag
            .attrs
            .iter()
            .filter(|a| !a.name.eq_ignore_ascii_case("da"))
        {
            let name = a.name.to_lowercase();
            if name == CONTENT {
                return Err(self.fail(
                    Fail::new(
                        a.pos,
                        "'cuntenuto' è chello ca sta tra <chiamm> e </chiamm>.",
                    )
                    .hint("Chiamma stu parametro n'ata manera."),
                ));
            }
            if values.iter().any(|(n, _, _)| *n == name) {
                return Err(self.fail(Fail::new(
                    a.pos,
                    format!("'{}' l'hê scritto doje vote.", a.name),
                )));
            }
            values.push((name, a.value.clone().unwrap_or_default(), a.pos));
        }
        let path = join(self.file(), &da);
        if self.chain.contains(&path) {
            let cycle = [self.chain.as_slice(), std::slice::from_ref(&path)].concat();
            return Err(self.fail(
                Fail::new(pos, format!("'{da}' se chiamma da sulo.")).hint(cycle.join(" → ")),
            ));
        }
        let source = self.read(&path, &da, pos)?;
        self.pieces.push(Piece {
            path: da.clone(),
            values,
            content,
            used: Vec::new(),
            caller: self.file().to_string(),
            at: tag.pos,
        });
        self.chain.push(path);
        let html = self.fragment(&source)?;
        self.chain.pop();
        let piece = self.pieces.pop().expect("pushed above");
        // A value nobody uses is a typo or a leftover: say so where it is written
        if let Some((name, _, at)) = piece
            .values
            .iter()
            .find(|(n, _, _)| !piece.used.contains(n))
        {
            return Err(self.fail(
                Fail::new(*at, format!("'{name}' nun 'o usa nisciuno int'a '{da}'."))
                    .hint(format!("Int'o pezzo scrive {{{{{name}}}}} addò 'o vuò.")),
            ));
        }
        let content_lost = piece.content.as_ref().is_some_and(|c| !c.trim().is_empty())
            && !piece.used.iter().any(|n| n == CONTENT);
        if content_lost {
            return Err(self.fail(
                Fail::new(
                    tag.pos,
                    format!("'{da}' nun tene {{{{cuntenuto}}}}: chello ca ce miette se perde."),
                )
                .hint("Miette {{cuntenuto}} int'o pezzo, o chiude subbeto: <chiamm … />."),
            ));
        }
        Ok(html.strip_suffix('\n').unwrap_or(&html).to_string())
    }

    /// The tokens up to the `</chiamm>` that closes `tag`, compiled here: they
    /// belong to this file, with its own placeholders.
    fn content(
        &mut self,
        tag: &Tag,
        tokens: &mut impl Iterator<Item = Token>,
    ) -> Result<String, Error> {
        let is_include = |name: &str| name.eq_ignore_ascii_case(INCLUDE);
        let mut inner = Vec::new();
        let mut depth = 0;
        loop {
            match tokens.next() {
                None => {
                    return Err(self.fail(
                        Fail::new(tag.pos, format!("<{}> nun l'hê maje chiuso.", tag.name))
                            .hint(format!("Scrive </{}> quanno fernesce.", tag.name)),
                    ))
                }
                Some(Token::End { name, .. }) if is_include(&name) && depth == 0 => break,
                Some(token) => {
                    match &token {
                        Token::Start(t) if is_include(&t.name) && !t.self_closing => depth += 1,
                        Token::End { name, .. } if is_include(name) => depth -= 1,
                        _ => {}
                    }
                    inner.push(token);
                }
            }
        }
        self.tokens(inner)
    }

    /// `{{nomme}}` in a piece called with parameters becomes the value given
    /// to `<chiamm>`, and `{{cuntenuto}}`, in text, what stood between its
    /// tags. Outside a piece the text stays as it is.
    fn fill(&mut self, text: &str, in_text: bool) -> Result<String, Error> {
        if self.pieces.is_empty() || !text.contains("{{") {
            return Ok(text.to_string());
        }
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(start) = rest.find("{{") {
            let after = &rest[start + 2..];
            let Some(len) = after.find("}}") else {
                break;
            };
            out.push_str(&rest[..start]);
            let name = after[..len].trim().to_lowercase();
            let is_name = !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_alphanumeric() || "-_".contains(c));
            if is_name {
                out.push_str(&self.placeholder(&name, in_text)?);
            } else {
                out.push_str(&rest[start..start + 2 + len + 2]);
            }
            rest = &after[len + 2..];
        }
        out.push_str(rest);
        Ok(out)
    }

    fn placeholder(&mut self, name: &str, in_text: bool) -> Result<String, Error> {
        let piece = self.pieces.last_mut().expect("fill checks it");
        let value = if name == CONTENT {
            match (&piece.content, in_text) {
                (_, false) => {
                    return Err(piece.error(
                        format!(
                            "'{}' tene {{{{cuntenuto}}}} dint'a n'attributo.",
                            piece.path
                        ),
                        "{{cuntenuto}} va sulo int'o testo d'o pezzo.".to_string(),
                    ))
                }
                (Some(html), true) => html.clone(),
                (None, true) => {
                    return Err(piece.error(
                        format!("'{}' vo' nu cuntenuto.", piece.path),
                        format!("Scrive <chiamm da=\"{}\">…</chiamm>.", piece.path),
                    ))
                }
            }
        } else {
            match piece.values.iter().find(|(n, _, _)| n == name) {
                // The value is attribute text: in the page text a `<` would open a tag
                Some((_, v, _)) if in_text => v.replace('<', "&lt;"),
                Some((_, v, _)) => v.clone(),
                None => {
                    return Err(piece.error(
                        format!("'{}' vo' '{name}'.", piece.path),
                        format!(
                            "Passalo accussì: <chiamm da=\"{}\" {name}=\"…\" />",
                            piece.path
                        ),
                    ))
                }
            }
        };
        piece.used.push(name.to_string());
        Ok(value)
    }

    /// The tag with its attribute values filled in (see `fill`).
    fn fill_tag(&mut self, mut tag: Tag) -> Result<Tag, Error> {
        for a in &mut tag.attrs {
            if let Some(v) = &a.value {
                a.value = Some(self.fill(v, false)?);
            }
        }
        Ok(tag)
    }

    fn read(&mut self, path: &str, written: &str, pos: Pos) -> Result<String, Error> {
        let source = (self.options.read)(path).ok_or_else(|| {
            self.fail(
                Fail::new(pos, format!("Nun trovo '{written}'."))
                    .hint(format!("L'aggio cercato ccà: {path}")),
            )
        })?;
        self.dependencies.push(path.to_string());
        Ok(source)
    }
}

/// Compiles FratmScript that starts at `at` in `file`, moving error
/// positions there.
fn fratm(code: &str, file: &str, at: Pos) -> Result<fratm_core::CompileResult, Error> {
    let options = fratm_core::CompileOptions {
        filename: Some(file.to_string()),
        ..Default::default()
    };
    fratm_core::compile(code, options).map_err(|e| {
        let (line, column) = (e.line().unwrap_or(1), e.column().unwrap_or(1));
        let pos = if line == 1 {
            Pos {
                line: at.line,
                column: at.column + column - 1,
            }
        } else {
            Pos {
                line: at.line + line - 1,
                column,
            }
        };
        let message = e.to_string();
        // fratm puts its own "Riga X, colonna Y: " in front: ours is in the position
        let message = match message.split_once(": ") {
            Some((head, rest)) if head.starts_with("Riga ") => rest.to_string(),
            _ => message,
        };
        Error {
            message,
            file: file.to_string(),
            line: pos.line,
            column: pos.column,
            suggestion: fratm_core::errors::suggest(&e, code),
        }
    })
}

fn event_list() -> String {
    vocabolario::EVENTS
        .iter()
        .map(|(n, _, _)| *n)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Joins a path written in `file` to the directory of `file`.
fn join(file: &str, path: &str) -> String {
    if path.starts_with('/') {
        return path.to_string();
    }
    let mut parts: Vec<&str> = file.split('/').collect();
    parts.pop();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." if parts.last().is_some_and(|p| !p.is_empty() && *p != "..") => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

/// Compiled code that imports or exports has to run as a module.
fn is_module(js: &str) -> bool {
    js.lines().any(|l| {
        ["import", "export"].iter().any(|k| {
            l.strip_prefix(k)
                .is_some_and(|rest| rest.starts_with([' ', '{', '*', '"', '\'']))
        })
    })
}

/// An attribute value as written, safe inside double quotes.
fn quote(value: &str) -> String {
    value.replace('"', "&quot;")
}

/// Text (compiled JavaScript) as an attribute value.
fn escape_attr(text: &str) -> String {
    text.replace('&', "&amp;").replace('"', "&quot;")
}

/// The character references an author may use inside a handler.
fn decode(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// JavaScript inside `<script>`: a `</script` in a string would end it.
fn escape_script(js: &str) -> String {
    let lower = js.to_ascii_lowercase();
    let mut out = String::with_capacity(js.len());
    let mut last = 0;
    for (i, _) in lower.match_indices("</script") {
        out.push_str(&js[last..i]);
        out.push_str("<\\/");
        last = i + 2;
    }
    out.push_str(&js[last..]);
    out
}

fn is_page(html: &str) -> bool {
    let mut rest = html.trim_start();
    while let Some(comment) = rest.strip_prefix("<!--") {
        rest = comment
            .split_once("-->")
            .map_or("", |(_, r)| r)
            .trim_start();
    }
    rest.strip_prefix("<html")
        .is_some_and(|r| r.starts_with(['>', ' ', '\n', '\t', '\r']))
}

fn page(mut html: String) -> String {
    if !html.to_ascii_lowercase().contains("<meta charset=") {
        if let Some(at) = html
            .find("<head")
            .and_then(|i| Some(i + html[i..].find('>')? + 1))
        {
            let rest = &html[at..];
            let meta = match rest.strip_prefix('\n') {
                Some(r) => {
                    let indent: String =
                        r.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
                    format!("\n{indent}<meta charset=\"utf-8\">")
                }
                None => "<meta charset=\"utf-8\">".to_string(),
            };
            html.insert_str(at, &meta);
        }
    }
    format!("<!DOCTYPE html>\n{html}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn html(source: &str) -> String {
        compile(source, &Options::default())
            .unwrap_or_else(|e| panic!("{e}"))
            .html
    }

    fn error(source: &str) -> Error {
        compile(source, &Options::default()).unwrap_err()
    }

    fn with_files(
        files: &[(&str, &str)],
        source: &str,
        fratm_src: FratmSrc,
    ) -> Result<Output, Error> {
        let files: HashMap<String, String> = files
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let read = move |path: &str| files.get(path).cloned();
        compile(
            source,
            &Options {
                filename: "sito/index.emmurt",
                read: &read,
                fratm_src,
            },
        )
    }

    #[test]
    fn translates_tags_attributes_and_values() {
        assert_eq!(
            html("<citofono tipo=\"buttone\" classe='a \"b\"' stutato>Sforna</citofono>"),
            "<button type=\"button\" class=\"a &quot;b&quot;\" disabled>Sforna</button>"
        );
        assert_eq!(
            html("<modulo addo=\"/ordina\" metodo=\"manna\"><miett tipo=\"posta\" obbligatorio></modulo>"),
            "<form action=\"/ordina\" method=\"post\"><input type=\"email\" required></form>"
        );
        assert_eq!(
            html("<vattenn addo=\"/\" arape=nuova>Casa</vattenn>"),
            "<a href=\"/\" target=\"_blank\">Casa</a>"
        );
    }

    #[test]
    fn void_and_self_closing_elements() {
        assert_eq!(html("<acap><acap /><stacc/>"), "<br><br><hr>");
        assert_eq!(html("<cascett />"), "<div></div>");
    }

    #[test]
    fn data_aria_and_custom_elements_pass_through() {
        assert_eq!(
            html("<pizza-card gusto=\"margherita\" classe=\"c\" data-x=\"1\" aria-label=\"p\"></pizza-card>"),
            "<pizza-card gusto=\"margherita\" class=\"c\" data-x=\"1\" aria-label=\"p\"></pizza-card>"
        );
        assert_eq!(
            html("<disegno viewBox=\"0 0 1 1\" larghezza=\"10\"><circle r=\"1\"/></disegno>"),
            "<svg viewBox=\"0 0 1 1\" width=\"10\"><circle r=\"1\"/></svg>"
        );
    }

    #[test]
    fn raw_elements_keep_their_content() {
        assert_eq!(
            html("<cess>div > p { color: red }</cess><titolo>a <b></titolo>"),
            "<style>div > p { color: red }</style><title>a <b></title>"
        );
    }

    #[test]
    fn cess_is_css_in_neapolitan() {
        assert_eq!(
            html("<cess>citofono:passanno { culore: russo; spazio: 8px }</cess>"),
            "<style>button:hover { color: red; padding: 8px }</style>"
        );
        assert_eq!(
            html("<cascett cess=\"culore: russo; sfunno-culore: janco\"></cascett>"),
            "<div style=\"color: red; background-color: white\"></div>"
        );
        // Attribute names and states in selectors use the words of the attributes
        assert_eq!(
            html("<cess>miett[tipo]:stutato, miett::esempio { culore: grigio }</cess>"),
            "<style>input[type]:disabled, input::placeholder { color: gray }</style>"
        );
        let e = error("<cess>\n  cascett { culore russo }\n</cess>");
        assert_eq!(e.line, 2, "{e}");
    }

    #[test]
    fn pages_get_doctype_and_charset() {
        assert_eq!(
            html("<cristian lingua=\"nap\">\n<capa>\n  <titolo>Uè</titolo>\n</capa>\n</cristian>\n"),
            "<!DOCTYPE html>\n<html lang=\"nap\">\n<head>\n  <meta charset=\"utf-8\">\n  <title>Uè</title>\n</head>\n</html>\n"
        );
        let own = html("<cristian><capa><inciucc codifica=\"latin1\"></capa></cristian>");
        assert!(!own.contains("utf-8"), "{own}");
        assert!(!html("<cascett></cascett>").contains("DOCTYPE"));
    }

    #[test]
    fn fratm_blocks_compile_to_javascript() {
        assert_eq!(
            html("<fratm>\n  stamm a dì(\"Uè\")\n</fratm>"),
            "<script>\nconsole.log(\"Uè\");\n</script>"
        );
        assert_eq!(
            html("<fratm>chiamm { a } da \"./a.js\"</fratm>"),
            "<script type=\"module\">\nimport { a } from \"./a.js\";\n</script>"
        );
        assert_eq!(
            html("<fratm>stamm a dì(\"</script>\")</fratm>"),
            "<script>\nconsole.log(\"<\\/script>\");\n</script>"
        );
        // Another type is not FratmScript: the content stays as written
        assert_eq!(
            html("<fratm tipo=\"text/javascript\">var a = 1 < 2</fratm>"),
            "<script type=\"text/javascript\">var a = 1 < 2</script>"
        );
        assert_eq!(
            html("<fratm tipo=\"application/ld+json\">{\"@type\": \"Pizzeria\"}</fratm>"),
            "<script type=\"application/ld+json\">{\"@type\": \"Pizzeria\"}</script>"
        );
        assert_eq!(
            html("<fratm tipo=\"modulo\">chiamm { a } da \"./a.js\"</fratm>"),
            "<script type=\"module\">\nimport { a } from \"./a.js\";\n</script>"
        );
    }

    #[test]
    fn fratm_src_is_kept_or_inlined() {
        assert_eq!(
            html("<fratm da=\"main.fratm\"></fratm>"),
            "<script src=\"main.fratm\" type=\"module\"></script>"
        );
        assert_eq!(
            html("<fratm da=\"main.fratm\" /><pere></pere>"),
            "<script src=\"main.fratm\" type=\"module\"></script><footer></footer>"
        );
        // Only .fratm files are compiled: other scripts load as they are
        assert_eq!(
            html("<fratm da=\"lib.js\" aropp /><fratm da=\"https://cdn.example/x.js\" />"),
            "<script src=\"lib.js\" defer></script><script src=\"https://cdn.example/x.js\"></script>"
        );
        let out = with_files(
            &[("sito/main.fratm", "stamm a dì(1)")],
            "<fratm da=\"main.fratm\" />",
            FratmSrc::Inline,
        )
        .unwrap();
        assert_eq!(out.html, "<script>\nconsole.log(1);\n</script>");
        assert_eq!(out.dependencies, ["sito/main.fratm"]);
    }

    #[test]
    fn handlers_are_fratmscript() {
        assert_eq!(
            html("<citofono quanno-tuocche=\"si (a e b) { sforna(&quot;x&quot;) }\"></citofono>"),
            "<button onclick=\"if (a &amp;&amp; b) {\n  sforna(&quot;x&quot;);\n}\"></button>"
        );
    }

    #[test]
    fn includes_paste_other_files() {
        let files = [
            (
                "sito/pezzi/menu.emmurt",
                "<tomtom><chiamm da=\"voce.emmurt\" /></tomtom>\n",
            ),
            ("sito/pezzi/voce.emmurt", "<voce>Pizze</voce>"),
        ];
        let out = with_files(
            &files,
            "<chiamm da=\"pezzi/menu.emmurt\" />",
            FratmSrc::Keep,
        )
        .unwrap();
        assert_eq!(out.html, "<nav><li>Pizze</li></nav>");
        assert_eq!(
            out.dependencies,
            ["sito/pezzi/menu.emmurt", "sito/pezzi/voce.emmurt"]
        );
    }

    #[test]
    fn pieces_take_parameters_and_content() {
        let files = [
            (
                "sito/pezzi/pizza.emmurt",
                "<fattariello classe=\"{{classe}}\">\n  <luccolo>{{ nomme }}</luccolo>\n  {{cuntenuto}}\n  <miett tipo=\"{{tipo}}\">\n</fattariello>\n",
            ),
            ("sito/pezzi/menu.emmurt", "<list><chiamm da=\"voce.emmurt\" testo=\"{{prima}}\" /></list>"),
            ("sito/pezzi/voce.emmurt", "<voce>{{testo}}</voce>"),
        ];
        let source = "<chiamm da=\"pezzi/pizza.emmurt\" nomme=\"Pizza &amp; <birra>\" classe=\"carta\" tipo=\"fattimiei\">\n<chiacchier>Cu 'a bufala</chiacchier>\n</chiamm>";
        let out = with_files(&files, source, FratmSrc::Keep).unwrap();
        // Values go through the usual translation; a `<` in text is escaped
        assert_eq!(
            out.html,
            "<article class=\"carta\">\n  <h2>Pizza &amp; &lt;birra></h2>\n  \n<p>Cu 'a bufala</p>\n\n  <input type=\"password\">\n</article>"
        );
        // A piece passes its own values on to the pieces it calls
        let out = with_files(
            &files,
            "<chiamm da=\"pezzi/menu.emmurt\" prima=\"Margherita\" />",
            FratmSrc::Keep,
        )
        .unwrap();
        assert_eq!(out.html, "<ul><li>Margherita</li></ul>");
        // Outside a piece the braces are text
        assert_eq!(
            html("<chiacchier>{{nomme}}</chiacchier>"),
            "<p>{{nomme}}</p>"
        );
    }

    #[test]
    fn piece_errors_point_at_the_call() {
        let files = [
            ("sito/pezzi/voce.emmurt", "<voce>{{testo}}</voce>"),
            (
                "sito/pezzi/carta.emmurt",
                "<cascett>{{cuntenuto}}</cascett>",
            ),
        ];
        let e = with_files(
            &files,
            "\n<chiamm da=\"pezzi/voce.emmurt\" />",
            FratmSrc::Keep,
        )
        .unwrap_err();
        assert_eq!(
            (e.file.as_str(), e.line, e.column, e.message.as_str()),
            (
                "sito/index.emmurt",
                2,
                2,
                "'pezzi/voce.emmurt' vo' 'testo'."
            )
        );
        let e = with_files(
            &files,
            "<chiamm da=\"pezzi/voce.emmurt\" testo=\"a\" piccante=\"sì\" />",
            FratmSrc::Keep,
        )
        .unwrap_err();
        assert_eq!(
            (e.column, e.message.as_str()),
            (
                42,
                "'piccante' nun 'o usa nisciuno int'a 'pezzi/voce.emmurt'."
            )
        );
        let e = with_files(
            &files,
            "<chiamm da=\"pezzi/voce.emmurt\" testo=\"a\">Uè</chiamm>",
            FratmSrc::Keep,
        )
        .unwrap_err();
        assert!(e.message.contains("nun tene {{cuntenuto}}"), "{e}");
        let e = with_files(
            &files,
            "<chiamm da=\"pezzi/carta.emmurt\" />",
            FratmSrc::Keep,
        )
        .unwrap_err();
        assert_eq!(e.message, "'pezzi/carta.emmurt' vo' nu cuntenuto.");
        let e = with_files(
            &files,
            "<chiamm da=\"pezzi/carta.emmurt\">\n<cascett>\n</chiamm>",
            FratmSrc::Keep,
        )
        .unwrap_err();
        assert_eq!(
            (e.line, e.message.as_str()),
            (2, "<cascett> nun l'hê maje chiuso.")
        );
    }

    #[test]
    fn include_errors_name_the_included_file() {
        let files = [("sito/pere.emmurt", "\n<cascett>")];
        let e = with_files(&files, "<chiamm da=\"pere.emmurt\" />", FratmSrc::Keep).unwrap_err();
        assert_eq!(
            (e.file.as_str(), e.line, e.column),
            ("sito/pere.emmurt", 2, 2)
        );

        let e = with_files(&[], "<chiamm da=\"nada.emmurt\" />", FratmSrc::Keep).unwrap_err();
        assert_eq!(e.message, "Nun trovo 'nada.emmurt'.");

        let files = [("sito/a.emmurt", "<chiamm da=\"index.emmurt\" />")];
        let e = with_files(&files, "<chiamm da=\"a.emmurt\" />", FratmSrc::Keep).unwrap_err();
        assert_eq!(
            e.suggestion.as_deref(),
            Some("sito/index.emmurt → sito/a.emmurt → sito/index.emmurt")
        );
    }

    #[test]
    fn english_names_are_errors_with_a_suggestion() {
        let e = error("<div classe=\"menu\">");
        assert_eq!((e.line, e.column), (1, 2));
        assert_eq!(e.message, "<div> nun è napoletano.");
        assert_eq!(e.suggestion.as_deref(), Some("Vuoi dire <cascett>?"));

        let e = error("<cascett class=\"menu\">");
        assert_eq!(e.suggestion.as_deref(), Some("Vuoi dire 'classe'?"));
        let e = error("<citofono onclick=\"x()\">");
        assert_eq!(e.suggestion.as_deref(), Some("Vuoi dire 'quanno-tuocche'?"));
        let e = error("<miett tipo=\"password\">");
        assert_eq!(
            (e.column, e.suggestion.as_deref()),
            (14, Some("Vuoi dire tipo=\"fattimiei\"?"))
        );
        let e = error("<cascet>");
        assert_eq!(e.suggestion.as_deref(), Some("Vuoi dire <cascett>?"));
        let e = error("<cascett clase=\"a\">");
        assert_eq!(e.suggestion.as_deref(), Some("Vuoi dire 'classe'?"));
        // An English name that has a word stays an error on an untranslated tag too
        let e = error("<track src=\"it.vtt\">");
        assert_eq!(e.suggestion.as_deref(), Some("Vuoi dire 'da'?"));
    }

    #[test]
    fn html_names_without_a_word_pass_as_written() {
        assert_eq!(
            html("<modulo addo=\"/carica\" metodo=\"manna\" enctype=\"multipart/form-data\" novalidate></modulo>"),
            "<form action=\"/carica\" method=\"post\" enctype=\"multipart/form-data\" novalidate></form>"
        );
        assert_eq!(
            html("<filmato><track kind=\"subtitles\" srclang=\"it\" da=\"it.vtt\"></filmato>"),
            "<video><track kind=\"subtitles\" srclang=\"it\" src=\"it.vtt\"></video>"
        );
        // Event handlers without a word keep their JavaScript
        assert_eq!(
            html("<cascett onwheel=\"zoom(event)\"></cascett>"),
            "<div onwheel=\"zoom(event)\"></div>"
        );
        // Names that are not HTML are still errors
        assert_eq!(
            error("<cascett pizza=\"x\">").message,
            "<cascett> nun cunosce l'attributo 'pizza'."
        );
        assert_eq!(error("<pizza>").message, "<pizza> nun 'o cunosco.");
    }

    #[test]
    fn structure_errors() {
        let e = error("<cascett>\n  <scatulell>Uè\n</cascett>");
        assert_eq!((e.line, e.column), (3, 3));
        assert_eq!(
            e.message,
            "Hê chiuso </cascett> ma <scatulell> (riga 2) è ancora araputo."
        );
        let e = error("<cascett>\n<scatulell>");
        assert_eq!(
            (e.line, e.message.as_str()),
            (2, "<scatulell> nun l'hê maje chiuso.")
        );
        assert_eq!(
            error("</cascett>").message,
            "</cascett> chiude na cosa ca nun è maje stata araputa."
        );
        assert_eq!(
            error("<acap></acap>").message,
            "<acap> nun se chiude: è sempe vacante."
        );
        assert_eq!(
            error("<cascett classe=a classe=b>").message,
            "'classe' l'hê scritto doje vote."
        );
    }

    #[test]
    fn fratm_errors_point_into_the_page() {
        let e = error("<cascett>\n<fratm>\n  chist nome = 1\n</fratm>");
        assert_eq!((e.line, e.column), (3, 9));
        assert!(e.message.contains("chist"), "{}", e.message);
        let e = error("<citofono quanno-tuocche=\"chist x = 1\">");
        assert_eq!(e.line, 1);
        assert!(e.column >= 26, "{e}");
    }

    #[test]
    fn join_resolves_relative_paths() {
        assert_eq!(join("index.emmurt", "pezzi/a.emmurt"), "pezzi/a.emmurt");
        assert_eq!(join("a/b/c.emmurt", "../d.emmurt"), "a/d.emmurt");
        assert_eq!(join("/abs/c.emmurt", "./d.emmurt"), "/abs/d.emmurt");
        assert_eq!(join("c.emmurt", "../d.emmurt"), "../d.emmurt");
        assert_eq!(join("a/c.emmurt", "/d.emmurt"), "/d.emmurt");
    }
}
