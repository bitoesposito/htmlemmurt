//! The vocabulary. Elements, attributes with their enumerated values and
//! events come from `fratm_core::html`, shared with FratmScript's JSX, so a
//! word means the same thing in a page and in a component. This module adds
//! what only pages need, and the lookups the compiler uses.

pub use fratm_core::html::{applies, Attribute, ATTRIBUTES, EVENTS, TAGS};

/// HTML elements with no Neapolitan word yet. They pass as written, like
/// the attributes and events below; English names that have a word are
/// still errors. When a word enters `fratm_core::html`, its name leaves
/// these lists (a test checks).
pub const UNTRANSLATED_TAGS: &[&str] = &[
    "area", "bdi", "bdo", "col", "colgroup", "data", "dfn", "embed", "hgroup", "map", "math",
    "menu", "object", "rp", "rt", "ruby", "samp", "search", "track", "var", "wbr",
];

/// HTML attributes with no Neapolitan word yet: they pass as written.
pub const UNTRANSLATED_ATTRIBUTES: &[&str] = &[
    "abbr",
    "accept-charset",
    "accesskey",
    "allow",
    "allowfullscreen",
    "as",
    "autocapitalize",
    "autocorrect",
    "blocking",
    "cite",
    "closedby",
    "color",
    "command",
    "commandfor",
    "coords",
    "data",
    "decoding",
    "default",
    "dir",
    "dirname",
    "enctype",
    "enterkeyhint",
    "fetchpriority",
    "form",
    "formaction",
    "formenctype",
    "formmethod",
    "formnovalidate",
    "formtarget",
    "headers",
    "high",
    "hreflang",
    "http-equiv",
    "imagesizes",
    "imagesrcset",
    "inert",
    "inputmode",
    "is",
    "ismap",
    "itemid",
    "itemprop",
    "itemref",
    "itemscope",
    "itemtype",
    "kind",
    "low",
    "media",
    "nomodule",
    "nonce",
    "novalidate",
    "optimum",
    "ping",
    "playsinline",
    "popover",
    "popovertarget",
    "popovertargetaction",
    "preload",
    "referrerpolicy",
    "reversed",
    "sandbox",
    "scope",
    "shadowrootclonable",
    "shadowrootdelegatesfocus",
    "shadowrootmode",
    "shadowrootserializable",
    "shape",
    "size",
    "sizes",
    "slot",
    "span",
    "srcdoc",
    "srclang",
    "srcset",
    "start",
    "translate",
    "usemap",
    "wrap",
    "writingsuggestions",
];

/// HTML event handlers with no Neapolitan word yet: they pass as written,
/// with their JavaScript.
pub const UNTRANSLATED_EVENTS: &[&str] = &[
    "onabort",
    "onafterprint",
    "onanimationcancel",
    "onanimationend",
    "onanimationiteration",
    "onanimationstart",
    "onauxclick",
    "onbeforeinput",
    "onbeforematch",
    "onbeforeprint",
    "onbeforetoggle",
    "onbeforeunload",
    "oncancel",
    "oncanplay",
    "oncanplaythrough",
    "onclose",
    "oncontextlost",
    "oncontextmenu",
    "oncontextrestored",
    "oncopy",
    "oncuechange",
    "oncut",
    "ondrag",
    "ondragend",
    "ondragenter",
    "ondragleave",
    "ondragover",
    "ondragstart",
    "ondrop",
    "ondurationchange",
    "onemptied",
    "onended",
    "onerror",
    "onfocusin",
    "onfocusout",
    "onformdata",
    "onhashchange",
    "oninvalid",
    "onkeypress",
    "onlanguagechange",
    "onloadeddata",
    "onloadedmetadata",
    "onloadstart",
    "onmessage",
    "onmessageerror",
    "onmousedown",
    "onmousemove",
    "onmouseout",
    "onmouseover",
    "onmouseup",
    "onoffline",
    "ononline",
    "onpagehide",
    "onpagereveal",
    "onpageshow",
    "onpageswap",
    "onpaste",
    "onpause",
    "onplay",
    "onplaying",
    "onpointercancel",
    "onpointerdown",
    "onpointerenter",
    "onpointerleave",
    "onpointermove",
    "onpointerout",
    "onpointerover",
    "onpointerup",
    "onpopstate",
    "onprogress",
    "onratechange",
    "onreset",
    "onresize",
    "onscrollend",
    "onsecuritypolicyviolation",
    "onseeked",
    "onseeking",
    "onselect",
    "onselectionchange",
    "onselectstart",
    "onslotchange",
    "onstalled",
    "onstorage",
    "onsuspend",
    "ontimeupdate",
    "ontoggle",
    "ontouchcancel",
    "ontouchend",
    "ontouchmove",
    "ontouchstart",
    "ontransitioncancel",
    "ontransitionend",
    "ontransitionrun",
    "ontransitionstart",
    "onunhandledrejection",
    "onunload",
    "onvolumechange",
    "onwaiting",
    "onwheel",
];

/// The include tag, resolved at compile time: `<chiamm da="pezzi/menu.emmurt" />`.
pub const INCLUDE: &str = "chiamm";

/// HTML elements that never have content or a closing tag.
pub const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

/// HTML elements whose content is not markup: it is copied as written, but
/// `<fratm>` content is compiled to JavaScript and `<cess>` content is
/// translated from Neapolitan CSS.
pub const RAW: &[&str] = &["script", "style", "title", "textarea", "svg"];

pub fn tag(nap: &str) -> Option<&'static str> {
    TAGS.iter().find(|(n, _, _)| *n == nap).map(|(_, h, _)| *h)
}

pub fn tag_for_html(html: &str) -> Option<&'static str> {
    TAGS.iter().find(|(_, h, _)| *h == html).map(|(n, _, _)| *n)
}

/// Raw-text elements, by Neapolitan name: the lexer reads their content as text.
pub fn is_raw(nap: &str) -> bool {
    tag(nap).is_some_and(|html| RAW.contains(&html))
}

pub fn attr_for(nap: &str, element: &str) -> Option<&'static Attribute> {
    ATTRIBUTES
        .iter()
        .find(|a| a.nap == nap && applies(a, element))
}

pub fn attr_for_html(html: &str, element: &str) -> Option<&'static Attribute> {
    ATTRIBUTES
        .iter()
        .find(|a| a.html == html && applies(a, element))
}

/// The HTML handler attribute (`onclick`) of a Neapolitan event.
pub fn event(nap: &str) -> Option<&'static str> {
    EVENTS
        .iter()
        .find(|(n, _, _)| *n == nap)
        .map(|(_, h, _)| *h)
}

pub fn event_for_html(html: &str) -> Option<&'static str> {
    EVENTS
        .iter()
        .find(|(_, h, _)| *h == html)
        .map(|(n, _, _)| *n)
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
    fn pages_and_html_names_do_not_clash() {
        for (nap, html, _) in TAGS {
            assert!(
                !UNTRANSLATED_TAGS.contains(html),
                "<{html}> has a word now: take it out of UNTRANSLATED_TAGS"
            );
            assert_ne!(*nap, INCLUDE, "<{nap}> is the include tag");
        }
        for a in ATTRIBUTES {
            assert!(
                !UNTRANSLATED_ATTRIBUTES.contains(&a.html),
                "'{}' has a word now: take it out of UNTRANSLATED_ATTRIBUTES",
                a.html
            );
            assert!(
                !UNTRANSLATED_ATTRIBUTES.contains(&a.nap),
                "'{}' is a word",
                a.nap
            );
        }
        for (_, html, _) in EVENTS {
            assert!(
                !UNTRANSLATED_EVENTS.contains(html),
                "'{html}' has a word now: take it out of UNTRANSLATED_EVENTS"
            );
        }
        // A Neapolitan attribute equal to an HTML one would make strict mode ambiguous
        for a in ATTRIBUTES {
            assert!(
                attr_for_html(a.nap, a.only.first().copied().unwrap_or("div"))
                    .is_none_or(|b| b.nap == a.nap),
                "attribute '{}' is also an HTML name",
                a.nap
            );
        }
    }

    #[test]
    fn element_specific_attributes_win() {
        assert_eq!(attr_for("addo", "form").unwrap().html, "action");
        assert_eq!(attr_for("addo", "a").unwrap().html, "href");
        assert!(attr_for("tipo", "input")
            .unwrap()
            .values
            .contains(&("fattimiei", "password")));
        assert!(attr_for("tipo", "ol").unwrap().values.is_empty());
        assert_eq!(event("quanno-tuocche"), Some("onclick"));
        assert_eq!(event_for_html("oninput"), Some("quanno-scrive"));
    }

    #[test]
    fn closest_finds_typos_only() {
        let names = || TAGS.iter().map(|(n, _, _)| *n);
        assert_eq!(closest("cascet", names()), Some("cascett"));
        assert_eq!(closest("citofno", names()), Some("citofono"));
        assert_eq!(closest("luccolon", names()), Some("luccolone"));
        assert_eq!(closest("pizza", names()), None);
    }
}
