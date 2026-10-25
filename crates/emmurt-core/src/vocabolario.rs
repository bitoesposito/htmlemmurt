//! The vocabulary: Neapolitan names of HTML elements, attributes, events and
//! enumerated attribute values. One table per kind, looked up in order, so
//! when two Neapolitan words map to the same HTML name the first one is the
//! one suggested in errors.

use serde::Serialize;

/// Element names: (Neapolitan, HTML).
pub const TAGS: &[(&str, &str)] = &[
    // Document
    ("cristian", "html"),
    ("capa", "head"),
    ("panz", "body"),
    ("titulo", "title"),
    ("inciucc", "meta"),
    ("attacca", "link"),
    ("basa", "base"),
    ("cess", "style"),
    ("fratm", "script"),
    ("copione", "script"),
    ("senzafratm", "noscript"),
    // Sections
    ("cappiell", "header"),
    ("pere", "footer"),
    ("tomtom", "nav"),
    ("core", "main"),
    ("piezz", "section"),
    ("fattariello", "article"),
    ("allato", "aside"),
    ("indirizzo", "address"),
    ("luccolone", "h1"),
    ("luccolo", "h2"),
    ("luccoliello", "h3"),
    ("luccolillo", "h4"),
    ("luccolino", "h5"),
    ("luccolicchio", "h6"),
    // Grouping
    ("cascett", "div"),
    ("chiacchier", "p"),
    ("stacc", "hr"),
    ("accussì", "pre"),
    ("accussi", "pre"),
    ("citazione", "blockquote"),
    ("list", "ul"),
    ("elenco", "ol"),
    ("voce", "li"),
    ("vocabolario", "dl"),
    ("parola", "dt"),
    ("significato", "dd"),
    ("quadro", "figure"),
    ("didascalia", "figcaption"),
    // Text
    ("vattenn", "a"),
    ("scatulell", "span"),
    ("acap", "br"),
    ("fort", "strong"),
    ("calcato", "em"),
    ("guapp", "b"),
    ("stuorto", "i"),
    ("sottolineato", "u"),
    ("cancellato", "s"),
    ("piccirill", "small"),
    ("evidenziato", "mark"),
    ("codice", "code"),
    ("fonte", "cite"),
    ("virgulette", "q"),
    ("abbreviazione", "abbr"),
    ("tiempo", "time"),
    ("sotto", "sub"),
    ("ncoppa", "sup"),
    ("tastiera", "kbd"),
    ("levato", "del"),
    ("aggiunto", "ins"),
    // Media
    ("cartulin", "img"),
    ("cornice", "picture"),
    ("funtana", "source"),
    ("filmato", "video"),
    ("suono", "audio"),
    ("fenestella", "iframe"),
    ("tela", "canvas"),
    ("disegno", "svg"),
    // Tables
    ("tavulat", "table"),
    ("cartello", "caption"),
    ("capotavula", "thead"),
    ("cuorpotavula", "tbody"),
    ("peretavula", "tfoot"),
    ("fila", "tr"),
    ("casella", "td"),
    ("capocasella", "th"),
    // Forms
    ("modulo", "form"),
    ("miett", "input"),
    ("citofono", "button"),
    ("targhett", "label"),
    ("scegli", "select"),
    ("scelta", "option"),
    ("gruppo", "optgroup"),
    ("ragionamento", "textarea"),
    ("recinto", "fieldset"),
    ("leggenda", "legend"),
    ("suggerimenti", "datalist"),
    ("risultato", "output"),
    ("avanzamento", "progress"),
    ("misura", "meter"),
    // Interactive and components
    ("spoiler", "details"),
    ("anteprim", "summary"),
    ("dialogo", "dialog"),
    ("stampo", "template"),
    ("fessura", "slot"),
];

/// HTML elements with no Neapolitan name yet: strict mode rejects them with
/// a message of their own instead of "unknown tag".
pub const UNTRANSLATED: &[&str] = &[
    "area", "bdi", "bdo", "col", "colgroup", "data", "dfn", "embed", "hgroup", "map", "math",
    "menu", "object", "ruby", "rp", "rt", "samp", "search", "track", "var", "wbr",
];

/// The include tag, resolved at compile time: `<chiamm da="pezzi/menu.emmurt" />`.
pub const INCLUDE: &str = "chiamm";

/// HTML elements that never have content or a closing tag.
pub const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

/// HTML elements whose content is not markup: it is copied as written
/// (`<fratm>` content is compiled to JavaScript instead).
pub const RAW: &[&str] = &["script", "style", "title", "textarea", "svg"];

/// An attribute. `only` lists the HTML elements it applies to (empty: all of
/// them); `values` translates enumerated values, (Neapolitan, HTML).
#[derive(Debug, Serialize)]
pub struct Attr {
    pub nap: &'static str,
    pub html: &'static str,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub only: &'static [&'static str],
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub values: &'static [(&'static str, &'static str)],
}

const fn attr(nap: &'static str, html: &'static str) -> Attr {
    Attr {
        nap,
        html,
        only: &[],
        values: &[],
    }
}

const BOOLEANS: &[(&str, &str)] = &[("overo", "true"), ("sfòls", "false"), ("sfols", "false")];

const INPUT_TYPES: &[(&str, &str)] = &[
    ("testo", "text"),
    ("fattimiei", "password"),
    ("segreto", "password"),
    ("posta", "email"),
    ("numero", "number"),
    ("spunta", "checkbox"),
    ("pallino", "radio"),
    ("manna", "submit"),
    ("azzera", "reset"),
    ("buttone", "button"),
    ("carta", "file"),
    ("data", "date"),
    ("ora", "time"),
    ("dataeora", "datetime-local"),
    ("mese", "month"),
    ("settimana", "week"),
    ("colore", "color"),
    ("cursore", "range"),
    ("cerca", "search"),
    ("telefono", "tel"),
    ("indirizzo", "url"),
    ("annascuso", "hidden"),
    ("cartulin", "image"),
];

const BUTTON_TYPES: &[(&str, &str)] = &[
    ("manna", "submit"),
    ("azzera", "reset"),
    ("buttone", "button"),
];

/// Attributes, element-specific entries before the general ones.
pub const ATTRS: &[Attr] = &[
    // Element-specific
    Attr {
        only: &["form"],
        ..attr("addo", "action")
    },
    Attr {
        only: &["input"],
        values: INPUT_TYPES,
        ..attr("tipo", "type")
    },
    Attr {
        only: &["button"],
        values: BUTTON_TYPES,
        ..attr("tipo", "type")
    },
    Attr {
        only: &["script"],
        values: &[("modulo", "module")],
        ..attr("tipo", "type")
    },
    // Global
    attr("classe", "class"),
    attr("chi", "id"),
    attr("cess", "style"),
    attr("titulo", "title"),
    attr("lengua", "lang"),
    attr("annascuso", "hidden"),
    attr("turno", "tabindex"),
    attr("ruolo", "role"),
    Attr {
        values: BOOLEANS,
        ..attr("scrivibile", "contenteditable")
    },
    Attr {
        values: BOOLEANS,
        ..attr("trascinabile", "draggable")
    },
    Attr {
        values: BOOLEANS,
        ..attr("correggi", "spellcheck")
    },
    // Links and resources
    attr("addo", "href"),
    attr("da", "src"),
    attr("spiegazione", "alt"),
    attr("larghezza", "width"),
    attr("altezza", "height"),
    Attr {
        values: &[
            ("nuova", "_blank"),
            ("ccà", "_self"),
            ("cca", "_self"),
            ("pate", "_parent"),
            ("ncoppa", "_top"),
        ],
        ..attr("arape", "target")
    },
    Attr {
        values: &[
            ("stile", "stylesheet"),
            ("icona", "icon"),
            ("manifesto", "manifest"),
            ("precarica", "preload"),
        ],
        ..attr("rapporto", "rel")
    },
    attr("scarica", "download"),
    attr("tipo", "type"),
    attr("codifica", "charset"),
    attr("nomme", "name"),
    attr("cuntenuto", "content"),
    Attr {
        values: &[("chianochiano", "lazy"), ("subbeto", "eager")],
        ..attr("carica", "loading")
    },
    attr("aropp", "defer"),
    attr("ampressa", "async"),
    // Forms
    attr("valore", "value"),
    attr("esempio", "placeholder"),
    attr("stutato", "disabled"),
    attr("spuntato", "checked"),
    attr("scigliuto", "selected"),
    attr("poffozza", "required"),
    attr("nunsetocca", "readonly"),
    attr("assaje", "multiple"),
    attr("subbeto", "autofocus"),
    attr("minimo", "min"),
    attr("massimo", "max"),
    attr("passo", "step"),
    attr("lunghezzaminima", "minlength"),
    attr("lunghezzamassima", "maxlength"),
    attr("modello", "pattern"),
    attr("pe", "for"),
    attr("suggerimenti", "list"),
    attr("accetta", "accept"),
    attr("etichetta", "label"),
    attr("righe", "rows"),
    attr("colonne", "cols"),
    Attr {
        values: &[("piglia", "get"), ("manna", "post"), ("dialogo", "dialog")],
        ..attr("metodo", "method")
    },
    Attr {
        values: &[("sì", "on"), ("si", "on"), ("no", "off")],
        ..attr("ricorda", "autocomplete")
    },
    // Tables, media, interactive
    attr("allarga", "colspan"),
    attr("allonga", "rowspan"),
    attr("partesubbeto", "autoplay"),
    attr("comandi", "controls"),
    attr("ripete", "loop"),
    attr("muto", "muted"),
    attr("copertina", "poster"),
    attr("quanno", "datetime"),
    attr("araputo", "open"),
];

/// Event handler attributes: (Neapolitan, HTML). Their value is FratmScript.
pub const EVENTS: &[(&str, &str)] = &[
    ("quanno-tuocche", "onclick"),
    ("quanno-tuocche-doje-vote", "ondblclick"),
    ("quanno-scrive", "oninput"),
    ("quanno-cagne", "onchange"),
    ("quanno-manne", "onsubmit"),
    ("quanno-carica", "onload"),
    ("quanno-premi", "onkeydown"),
    ("quanno-aize", "onkeyup"),
    ("quanno-trase", "onmouseenter"),
    ("quanno-esce", "onmouseleave"),
    ("quanno-piglie", "onfocus"),
    ("quanno-perde", "onblur"),
];

pub fn tag(nap: &str) -> Option<&'static str> {
    TAGS.iter().find(|(n, _)| *n == nap).map(|(_, h)| *h)
}

pub fn tag_for_html(html: &str) -> Option<&'static str> {
    TAGS.iter().find(|(_, h)| *h == html).map(|(n, _)| *n)
}

/// Raw-text elements, by Neapolitan name: the lexer reads their content as text.
pub fn is_raw(nap: &str) -> bool {
    tag(nap).is_some_and(|html| RAW.contains(&html))
}

fn applies(a: &Attr, element: &str) -> bool {
    a.only.is_empty() || a.only.contains(&element)
}

pub fn attr_for(nap: &str, element: &str) -> Option<&'static Attr> {
    ATTRS.iter().find(|a| a.nap == nap && applies(a, element))
}

pub fn attr_for_html(html: &str, element: &str) -> Option<&'static Attr> {
    ATTRS.iter().find(|a| a.html == html && applies(a, element))
}

pub fn event(nap: &str) -> Option<&'static str> {
    EVENTS.iter().find(|(n, _)| *n == nap).map(|(_, h)| *h)
}

pub fn event_for_html(html: &str) -> Option<&'static str> {
    EVENTS.iter().find(|(_, h)| *h == html).map(|(n, _)| *n)
}

/// The candidate closest to `word`, if it is a plausible typo of it.
pub fn closest<'a>(word: &str, candidates: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let limit = (word.chars().count() / 3).clamp(1, 2);
    candidates
        .map(|c| (distance(word, c), c))
        .filter(|(d, _)| *d <= limit)
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}

/// Levenshtein distance, on chars.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cur = row[j + 1];
            row[j + 1] = if ca == *cb {
                prev
            } else {
                1 + prev.min(row[j]).min(cur)
            };
            prev = cur;
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neapolitan_names_are_unique_and_distinct_from_html() {
        for (i, (nap, html)) in TAGS.iter().enumerate() {
            assert!(
                !TAGS[..i].iter().any(|(n, _)| n == nap),
                "<{nap}> twice in TAGS"
            );
            // A Neapolitan name equal to an HTML one would make strict mode ambiguous
            assert!(tag_for_html(nap).is_none(), "<{nap}> is also an HTML name");
            assert!(!nap.contains('-'), "<{nap}> would read as a custom element");
            assert!(!UNTRANSLATED.contains(html), "<{html}> is in UNTRANSLATED");
        }
        for a in ATTRS {
            assert!(
                attr_for_html(a.nap, a.only.first().copied().unwrap_or("div"))
                    .is_none_or(|b| b.nap == a.nap),
                "attribute '{}' is also an HTML name",
                a.nap
            );
            for (nap, _) in a.values {
                assert!(
                    !a.values.iter().any(|(_, h)| h == nap),
                    "value '{nap}' of '{}' is also an HTML value",
                    a.nap
                );
            }
        }
    }

    #[test]
    fn element_specific_attributes_win() {
        assert_eq!(attr_for("addo", "form").unwrap().html, "action");
        assert_eq!(attr_for("addo", "a").unwrap().html, "href");
        assert_eq!(attr_for("tipo", "input").unwrap().values, INPUT_TYPES);
        assert!(attr_for("tipo", "ol").unwrap().values.is_empty());
    }

    #[test]
    fn closest_finds_typos_only() {
        let names = || TAGS.iter().map(|(n, _)| *n);
        assert_eq!(closest("cascet", names()), Some("cascett"));
        assert_eq!(closest("citofno", names()), Some("citofono"));
        assert_eq!(closest("luccolon", names()), Some("luccolone"));
        assert_eq!(closest("pizza", names()), None);
    }
}
