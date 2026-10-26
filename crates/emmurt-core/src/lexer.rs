//! Splits htmlemmurt source into text, comments, tags and the raw content of
//! elements like `<fratm>` and `<cess>`. Names are kept as written; the
//! compiler translates and checks them.

use crate::vocabolario;
use crate::Fail;

/// A 1-based position. Columns count chars.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    /// Text and comments, copied as written.
    Text(String),
    Start(Tag),
    End {
        name: String,
        pos: Pos,
    },
    /// Content of a raw-text element; `pos` is where it starts.
    Raw {
        text: String,
        pos: Pos,
    },
}

#[derive(Debug, PartialEq)]
pub struct Tag {
    pub name: String,
    pub attrs: Vec<Attr>,
    pub self_closing: bool,
    /// Position of the name, right after `<`.
    pub pos: Pos,
}

#[derive(Debug, PartialEq)]
pub struct Attr {
    pub name: String,
    pub value: Option<String>,
    pub pos: Pos,
    /// Position of the first char of the value (inside the quotes).
    pub value_pos: Pos,
}

struct Lexer {
    chars: Vec<char>,
    i: usize,
    line: usize,
    column: usize,
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, Fail> {
    let mut lx = Lexer {
        chars: source.chars().collect(),
        i: 0,
        line: 1,
        column: 1,
    };
    let mut tokens = Vec::new();
    let mut text = String::new();
    while let Some(c) = lx.peek(0) {
        if c != '<' {
            text.push(lx.bump());
        } else if lx.starts_with("<!--") {
            let pos = lx.pos();
            let Some(len) = lx.find("-->") else {
                return Err(Fail::new(pos, "Stu commento nun se chiude: manca '-->'."));
            };
            for _ in 0..len + 3 {
                text.push(lx.bump());
            }
        } else if lx.starts_with("<!") {
            return Err(Fail::new(lx.pos(), "<!DOCTYPE> nun serve.")
                .hint("Quanno scrive <cristian> 'o metto io."));
        } else if lx.peek(1) == Some('/') && lx.peek(2).is_some_and(char::is_alphabetic) {
            flush(&mut text, &mut tokens);
            lx.bump();
            lx.bump();
            let pos = lx.pos();
            let name = lx.name();
            lx.skip_space();
            if lx.peek(0) != Some('>') {
                return Err(unclosed_tag(lx.pos()));
            }
            lx.bump();
            tokens.push(Token::End { name, pos });
        } else if lx.peek(1).is_some_and(char::is_alphabetic) {
            flush(&mut text, &mut tokens);
            lx.bump();
            let tag = lx.tag()?;
            let raw = !tag.self_closing && vocabolario::is_raw(&tag.name.to_lowercase());
            let (name, pos) = (tag.name.clone(), tag.pos);
            tokens.push(Token::Start(tag));
            if raw {
                let start = lx.pos();
                let Some(len) = lx.find_close(&name) else {
                    return Err(Fail::new(pos, format!("<{name}> nun l'hê maje chiuso."))
                        .hint(format!("Scrive </{name}> quanno fernesce.")));
                };
                let text = (0..len).map(|_| lx.bump()).collect();
                tokens.push(Token::Raw { text, pos: start });
            }
        } else {
            // A lone `<`, as in "a < b": plain text, as in HTML
            text.push(lx.bump());
        }
    }
    flush(&mut text, &mut tokens);
    Ok(tokens)
}

fn flush(text: &mut String, tokens: &mut Vec<Token>) {
    if !text.is_empty() {
        tokens.push(Token::Text(std::mem::take(text)));
    }
}

fn unclosed_tag(pos: Pos) -> Fail {
    Fail::new(pos, "Stu tag nun se chiude: ccà me spettavo '>'.")
}

impl Lexer {
    fn peek(&self, ahead: usize) -> Option<char> {
        self.chars.get(self.i + ahead).copied()
    }

    fn pos(&self) -> Pos {
        Pos {
            line: self.line,
            column: self.column,
        }
    }

    fn bump(&mut self) -> char {
        let c = self.chars[self.i];
        self.i += 1;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        c
    }

    fn starts_with(&self, s: &str) -> bool {
        s.chars().enumerate().all(|(k, c)| self.peek(k) == Some(c))
    }

    /// Chars from here to the next `s`.
    fn find(&self, s: &str) -> Option<usize> {
        let s: Vec<char> = s.chars().collect();
        self.chars[self.i..].windows(s.len()).position(|w| w == s)
    }

    /// Chars from here to the closing tag of a raw element: `</name`
    /// followed by a space or `>`, in any case.
    fn find_close(&self, name: &str) -> Option<usize> {
        let close: Vec<char> = format!("</{}", name.to_lowercase()).chars().collect();
        (self.i..self.chars.len()).find_map(|at| {
            let candidate = self.chars.get(at..at + close.len())?;
            let matches = candidate
                .iter()
                .zip(&close)
                .all(|(a, b)| a.to_lowercase().eq(b.to_lowercase()));
            let next = self.chars.get(at + close.len());
            (matches && next.is_some_and(|c| *c == '>' || c.is_whitespace())).then(|| at - self.i)
        })
    }

    fn skip_space(&mut self) {
        while self.peek(0).is_some_and(char::is_whitespace) {
            self.bump();
        }
    }

    /// A tag or attribute name: everything up to a space, `/`, `>`, `=`, quote or `<`.
    fn name(&mut self) -> String {
        let mut name = String::new();
        while let Some(c) = self.peek(0) {
            if c.is_whitespace() || matches!(c, '/' | '>' | '=' | '"' | '\'' | '<') {
                break;
            }
            name.push(self.bump());
        }
        name
    }

    /// The rest of a start tag, after `<`.
    fn tag(&mut self) -> Result<Tag, Fail> {
        let pos = self.pos();
        let name = self.name();
        let mut attrs = Vec::new();
        loop {
            self.skip_space();
            let self_closing = self.starts_with("/>");
            if self_closing {
                self.bump();
            }
            match self.peek(0) {
                None | Some('<') => return Err(unclosed_tag(self.pos())),
                Some('>') => {
                    self.bump();
                    return Ok(Tag {
                        name,
                        attrs,
                        self_closing,
                        pos,
                    });
                }
                Some(_) => attrs.push(self.attr()?),
            }
        }
    }

    fn attr(&mut self) -> Result<Attr, Fail> {
        let pos = self.pos();
        let name = self.name();
        if name.is_empty() {
            let c = self.peek(0).unwrap_or(' ');
            return Err(Fail::new(pos, format!("Ccà nun me spettavo '{c}'."))
                .hint("Dint'a nu tag ce vanno sulo attribute: nomme=\"valore\"."));
        }
        self.skip_space();
        if self.peek(0) != Some('=') {
            return Ok(Attr {
                name,
                value: None,
                pos,
                value_pos: pos,
            });
        }
        self.bump();
        self.skip_space();
        let open = self.pos();
        let quote = self.peek(0).filter(|c| *c == '"' || *c == '\'');
        if quote.is_some() {
            self.bump();
        }
        let value_pos = self.pos();
        let mut value = String::new();
        loop {
            match (self.peek(0), quote) {
                (None, Some(_)) => {
                    return Err(Fail::new(
                        open,
                        "'E virgulette ca hê araputo ccà nun se chiudono maje.",
                    ))
                }
                (Some(c), Some(q)) if c == q => {
                    self.bump();
                    break;
                }
                (None, None) => break,
                (Some(c), None) if c.is_whitespace() || c == '>' => break,
                _ => value.push(self.bump()),
            }
        }
        Ok(Attr {
            name,
            value: Some(value),
            pos,
            value_pos,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(line: usize, column: usize) -> Pos {
        Pos { line, column }
    }

    #[test]
    fn tags_attributes_and_text() {
        let tokens =
            tokenize("<cascett classe=\"a\" chi='b' annascuso data-x=1>\n Uè</cascett>").unwrap();
        let Token::Start(tag) = &tokens[0] else {
            panic!()
        };
        assert_eq!(tag.name, "cascett");
        assert_eq!(tag.pos, p(1, 2));
        let attrs: Vec<_> = tag
            .attrs
            .iter()
            .map(|a| (a.name.as_str(), a.value.as_deref()))
            .collect();
        assert_eq!(
            attrs,
            [
                ("classe", Some("a")),
                ("chi", Some("b")),
                ("annascuso", None),
                ("data-x", Some("1"))
            ]
        );
        assert_eq!(tag.attrs[0].value_pos, p(1, 18));
        assert_eq!(tokens[1], Token::Text("\n Uè".into()));
        assert_eq!(
            tokens[2],
            Token::End {
                name: "cascett".into(),
                pos: p(2, 6)
            }
        );
    }

    #[test]
    fn raw_content_is_not_markup() {
        let tokens = tokenize("<fratm>si (a <b) { }</FRATM >").unwrap();
        assert_eq!(
            tokens[1],
            Token::Raw {
                text: "si (a <b) { }".into(),
                pos: p(1, 8)
            }
        );
        assert!(matches!(&tokens[2], Token::End { name, .. } if name == "FRATM"));
    }

    #[test]
    fn comments_and_lone_brackets_are_text() {
        let tokens = tokenize("a < b <!-- <cascett> -->").unwrap();
        assert_eq!(tokens, [Token::Text("a < b <!-- <cascett> -->".into())]);
    }

    #[test]
    fn errors_point_at_the_problem() {
        let err = tokenize("<cascett classe=\"a>").unwrap_err();
        assert_eq!(err.pos, p(1, 17));
        let err = tokenize("\n<fratm> tien x = 1").unwrap_err();
        assert_eq!(
            (err.pos, err.message.as_str()),
            (p(2, 2), "<fratm> nun l'hê maje chiuso.")
        );
        assert!(tokenize("<!-- ").is_err());
        assert!(tokenize("<!DOCTYPE html>").is_err());
        assert!(tokenize("<cascett").is_err());
    }
}
