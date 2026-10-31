//! The formatter behind `emmurt fmt`. It moves line breaks and spaces only,
//! so a formatted page compiles to the same page.

use crate::lexer::{self, Tag, Token};
use crate::vocabolario::{self, VOID};
use crate::{close, unclosed, Error, Open};

const INDENT: &str = "  ";
/// The spaces HTML ignores between words, line breaks aside.
const SPACE: [char; 4] = [' ', '\t', '\r', '\x0c'];

/// Formats a page or a piece: two spaces for each open element, one
/// attribute per line when a tag was written over more lines, no spaces at
/// the end of a line, at most one blank line in a row. Text, comments and
/// values keep their characters, and so does the content of `<accussì>` and
/// `<ragionamento>`, where spaces count. The content of `<fratm>`, `<cess>`
/// and the other raw elements moves as a block and keeps its own
/// indentation, except FratmScript with a template string, whose lines are
/// left as they are.
pub fn format(source: &str, filename: &str) -> Result<String, Error> {
    let crlf = source.contains("\r\n");
    let source = source.replace("\r\n", "\n");
    let tokens = lexer::tokenize(&source).map_err(|f| f.in_file(filename))?;
    let mut p = Printer::default();
    let mut open: Vec<Open> = Vec::new();
    // Open <accussì> elements: inside them every space counts
    let mut pre = 0;
    for token in tokens {
        match token {
            Token::Text(text) if pre > 0 => p.out.push_str(&text),
            Token::Text(text) => p.text(&text, open.len()),
            Token::Raw { text, .. } => {
                let html = open.last().map(|o| html_name(&o.name)).unwrap_or_default();
                if pre > 0 || html == "textarea" || (html == "script" && text.contains('`')) {
                    p.out.push_str(&text);
                } else {
                    p.block(&text, open.len());
                }
            }
            Token::Start(tag) => {
                let html = html_name(&tag.name);
                let text = start_tag(&tag, open.len());
                match pre {
                    0 => p.put(&text, open.len()),
                    _ => p.out.push_str(&text),
                }
                if !tag.self_closing && !VOID.contains(&html.as_str()) {
                    pre += usize::from(html == "pre");
                    open.push(Open {
                        name: tag.name,
                        pos: tag.pos,
                    });
                }
            }
            Token::End { name, pos } => {
                let html = html_name(&name);
                close(&mut open, &name, &html, pos).map_err(|f| f.in_file(filename))?;
                let text = format!("</{name}>");
                match pre {
                    0 => p.put(&text, open.len()),
                    _ => p.out.push_str(&text),
                }
                pre -= usize::from(html == "pre");
            }
        }
    }
    unclosed(&open).map_err(|f| f.in_file(filename))?;
    let mut out = p.out.trim_end_matches(SPACE).to_string();
    if !out.is_empty() {
        out.push('\n');
    }
    Ok(if crlf { out.replace('\n', "\r\n") } else { out })
}

/// The HTML name of an element, or the name as it is when it has no
/// Neapolitan word.
fn html_name(name: &str) -> String {
    let lower = name.to_lowercase();
    vocabolario::tag(&lower).map_or(lower, str::to_string)
}

/// A start tag on one line, or with one attribute per line when it was
/// written over more lines. Values keep their characters.
fn start_tag(tag: &Tag, level: usize) -> String {
    let lines = tag.attrs.iter().any(|a| a.pos.line != tag.pos.line);
    let gap = match lines {
        true => format!("\n{}", INDENT.repeat(level + 1)),
        false => " ".to_string(),
    };
    let mut out = format!("<{}", tag.name);
    for a in &tag.attrs {
        out.push_str(&gap);
        out.push_str(&a.name);
        if let Some(v) = &a.value {
            let quote = match (v.contains('"'), v.contains('\'')) {
                (false, _) => "\"",
                (true, false) => "'",
                // Written without quotes, and it cannot have any
                (true, true) => "",
            };
            out.push_str(&format!("={quote}{v}{quote}"));
        }
    }
    if lines {
        out.push('\n');
        out.push_str(&INDENT.repeat(level));
    }
    out.push_str(match (tag.self_closing, lines) {
        (false, _) => ">",
        (true, false) => " />",
        (true, true) => "/>",
    });
    out
}

#[derive(Default)]
struct Printer {
    out: String,
    /// Line breaks since the last thing written.
    breaks: usize,
}

impl Printer {
    fn at_line_start(&self) -> bool {
        self.breaks > 0 || self.out.is_empty()
    }

    /// Writes `text`, on a new line indented at `level` if there were line
    /// breaks before it.
    fn put(&mut self, text: &str, level: usize) {
        if self.breaks > 0 && !self.out.is_empty() {
            self.out.push_str(&"\n".repeat(self.breaks.min(2)));
            self.out.push_str(&INDENT.repeat(level));
        }
        self.breaks = 0;
        self.out.push_str(text);
    }

    fn newline(&mut self) {
        self.out.truncate(self.out.trim_end_matches(SPACE).len());
        self.breaks += 1;
    }

    /// Text between tags: each line at `level`, comments as a block.
    fn text(&mut self, text: &str, level: usize) {
        let mut rest = text;
        loop {
            let at = rest.find("<!--").unwrap_or(rest.len());
            for (i, line) in rest[..at].split('\n').enumerate() {
                if i > 0 {
                    self.newline();
                }
                let line = match self.at_line_start() {
                    true => line.trim_start_matches(SPACE),
                    false => line,
                };
                if !line.is_empty() {
                    self.put(line, level);
                }
            }
            if at == rest.len() {
                return;
            }
            let end = rest[at..].find("-->").map_or(rest.len(), |e| at + e + 3);
            self.block(&rest[at..end], level);
            rest = &rest[end..];
        }
    }

    /// Lines that move together: the first goes on where it is, the others
    /// keep their indentation relative to each other, the least indented at
    /// `level`.
    fn block(&mut self, text: &str, level: usize) {
        let mut lines = text.split('\n');
        let first = lines.next().unwrap_or_default();
        let first = match self.at_line_start() {
            true => first.trim_start_matches(SPACE),
            false => first,
        };
        if !first.is_empty() {
            self.put(first, level);
        }
        let rest: Vec<&str> = lines.collect();
        let indent = |l: &str| l.len() - l.trim_start_matches(SPACE).len();
        let blank = |l: &str| l.trim_matches(SPACE).is_empty();
        let margin = rest.iter().filter(|l| !blank(l)).map(|l| indent(l)).min();
        for line in rest {
            self.newline();
            if !blank(line) {
                self.put(&line[margin.unwrap_or(0)..], level);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt(source: &str) -> String {
        format(source, "x.emmurt").unwrap()
    }

    #[test]
    fn indents_by_nesting() {
        assert_eq!(
            fmt("\n\n<cascett>\n<chiacchier>Uè   \n<fort>bello</fort></chiacchier>\n\n\n\n<acap>\n<chiamm da=\"a.emmurt\"/>\n    </cascett>  "),
            "<cascett>\n  <chiacchier>Uè\n    <fort>bello</fort></chiacchier>\n\n  <acap>\n  <chiamm da=\"a.emmurt\" />\n</cascett>\n"
        );
    }

    #[test]
    fn keeps_spaces_inside_a_line() {
        assert_eq!(
            fmt("<chiacchier>Uè <fort>bello</fort> assaje</chiacchier>"),
            "<chiacchier>Uè <fort>bello</fort> assaje</chiacchier>\n"
        );
        assert_eq!(
            fmt("<cascett>a&nbsp;\u{a0}</cascett>"),
            "<cascett>a&nbsp;\u{a0}</cascett>\n"
        );
    }

    #[test]
    fn tags_over_more_lines_get_one_attribute_per_line() {
        assert_eq!(
            fmt("<cascett>\n<miett tipo=text nomme='a \"b\"'\n   esempio=\"c\" stutato>\n<cascett classe='x' />\n</cascett>"),
            "<cascett>\n  <miett\n    tipo=\"text\"\n    nomme='a \"b\"'\n    esempio=\"c\"\n    stutato\n  >\n  <cascett classe=\"x\" />\n</cascett>\n"
        );
    }

    #[test]
    fn spaces_that_count_stay() {
        let source = "<core>\n<accussì>\n  a   \n    <fort>b</fort>\n</accussì>\n<ragionamento>\n  x  \n</ragionamento>\n</core>";
        assert_eq!(
            fmt(source),
            "<core>\n  <accussì>\n  a   \n    <fort>b</fort>\n</accussì>\n  <ragionamento>\n  x  \n</ragionamento>\n</core>\n"
        );
    }

    #[test]
    fn scripts_styles_and_comments_move_as_a_block() {
        let source = "<panz>\n<fratm>\nsi (x) {\n    y()   \n}\n</fratm>\n<cess>a { b: c }</cess>\n<!--\n  nota\n-->\n</panz>";
        assert_eq!(
            fmt(source),
            "<panz>\n  <fratm>\n    si (x) {\n        y()\n    }\n  </fratm>\n  <cess>a { b: c }</cess>\n  <!--\n    nota\n  -->\n</panz>\n"
        );
        // Moving these lines would change the string
        let template = "<panz>\n<fratm>\nx = `a\n  b`\n</fratm>\n</panz>";
        assert_eq!(
            fmt(template),
            "<panz>\n  <fratm>\nx = `a\n  b`\n</fratm>\n</panz>\n"
        );
    }

    #[test]
    fn keeps_windows_line_endings() {
        assert_eq!(
            fmt("<core>\r\n<acap>\r\n</core>"),
            "<core>\r\n  <acap>\r\n</core>\r\n"
        );
    }

    #[test]
    fn broken_nesting_is_the_compiler_error() {
        let err = format("<core>\n<cascett>\n</core>", "x.emmurt").unwrap_err();
        assert_eq!(
            (err.line, err.message.as_str()),
            (
                3,
                "Hê chiuso </core> ma <cascett> (riga 2) è ancora araputo."
            )
        );
        let err = format("<core>", "x.emmurt").unwrap_err();
        assert_eq!(err.message, "<core> nun l'hê maje chiuso.");
    }

    /// Every example and golden case: formatting twice changes nothing, and
    /// only spaces and line breaks move.
    #[test]
    fn examples_keep_their_meaning() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
        let mut files = Vec::new();
        for dir in [
            "examples",
            "examples/pezzi",
            "crates/emmurt-core/tests/golden/cases",
        ] {
            for entry in std::fs::read_dir(format!("{root}/{dir}")).unwrap() {
                let path = entry.unwrap().path();
                let name = path.to_string_lossy().to_string();
                if name.ends_with(".emmurt") && !name.ends_with(".sbagliato.emmurt") {
                    files.push(path);
                }
            }
        }
        assert!(files.len() > 10);
        let squeeze = |s: &str| s.split_whitespace().collect::<String>();
        for path in files {
            let source = std::fs::read_to_string(&path).unwrap();
            let once = fmt(&source);
            assert_eq!(fmt(&once), once, "{}", path.display());
            assert_eq!(squeeze(&once), squeeze(&source), "{}", path.display());
        }
    }
}
