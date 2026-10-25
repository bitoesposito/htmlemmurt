//! WebAssembly bindings: the compiler for Node, Vite, the browser runtime
//! and the VS Code extension.

use emmurt_core::{vocabolario, Error, FratmSrc, Options};
use js_sys::Function;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct Response {
    ok: bool,
    /// HTML for `compile`, JavaScript for `compileFratm`.
    code: Option<String>,
    dependencies: Vec<String>,
    errors: Vec<Error>,
}

fn to_js(response: &Response) -> JsValue {
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    response.serialize(&serializer).unwrap_or(JsValue::NULL)
}

/// Compiles a page. `read(path)` returns the text of an included file, or
/// null when it does not exist. `inline` writes `<fratm da>` files into the
/// page instead of leaving them to a bundler.
#[wasm_bindgen]
pub fn compile(source: &str, filename: &str, read: Option<Function>, inline: bool) -> JsValue {
    // Anything but a string, or a throw, means the file is not there
    let read = |path: &str| -> Option<String> {
        read.as_ref()?
            .call1(&JsValue::NULL, &JsValue::from_str(path))
            .ok()?
            .as_string()
    };
    let options = Options {
        filename,
        read: &read,
        fratm_src: if inline {
            FratmSrc::Inline
        } else {
            FratmSrc::Keep
        },
    };
    to_js(&match emmurt_core::compile(source, &options) {
        Ok(out) => Response {
            ok: true,
            code: Some(out.html),
            dependencies: out.dependencies,
            errors: vec![],
        },
        Err(e) => Response {
            ok: false,
            code: None,
            dependencies: vec![],
            errors: vec![e],
        },
    })
}

/// Compiles a `.fratm` module to JavaScript.
#[wasm_bindgen(js_name = compileFratm)]
pub fn compile_fratm(source: &str, filename: &str) -> JsValue {
    to_js(&match emmurt_core::compile_fratm(source, filename) {
        Ok(code) => Response {
            ok: true,
            code: Some(code),
            dependencies: vec![],
            errors: vec![],
        },
        Err(e) => Response {
            ok: false,
            code: None,
            dependencies: vec![],
            errors: vec![e],
        },
    })
}

/// Tags, attributes and events, as JSON.
#[wasm_bindgen]
pub fn vocabulary() -> String {
    serde_json::json!({
        "tags": vocabolario::TAGS,
        "attributes": vocabolario::ATTRS,
        "events": vocabolario::EVENTS,
    })
    .to_string()
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// A JavaScript runtime error in Neapolitan, as FratmScript says it.
#[wasm_bindgen(js_name = runtimeError)]
pub fn runtime_error(name: &str, message: &str) -> String {
    fratm_core::runtime::napoletanize_runtime_error(name, message)
}
