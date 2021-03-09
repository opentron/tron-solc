use std::error::Error;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::sync::Mutex;

use lazy_static::lazy_static;
use rusty_v8 as v8;

use self::config::determine_config_directory;
pub use self::input::Input;
pub use self::output::Output;

mod config;
mod input;
mod output;

const SOLJSON_FILENAME: &'static str = "solidity-js_0.5.15_GreatVoyage_v4.1.js";
const SOLJSON_URL: &'static str =
    "https://github.com/tronprotocol/solidity/releases/download/tv_0.5.15/solidity-js_0.5.15_GreatVoyage_v4.1.js";

/*
const SOLJSON_FILENAME: &'static str = "solidity-js_0.6.0_GreatVoyage_v4.1.2.js";
const SOLJSON_URL: &'static str =
        "https://github.com/tronprotocol/solidity/releases/download/tv_0.6.0/solidity-js_0.6.0_GreatVoyage_v4.1.2.js";
*/

lazy_static! {
    static ref INIT_LOCK: Mutex<u32> = Mutex::new(0);
}

/// The `log` function in js.
fn debug_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut _retval: v8::ReturnValue,
) {
    let message = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);

    println!("D: {:?}", message);

    // retval.set(v8::String::new(scope, "test").unwrap().into());
}

fn translate_import_url(path: &str) -> String {
    if path.starts_with("@openzeppelin-contracts/") {
        path.replace(
            "@openzeppelin-contracts/",
            "https://raw.githubusercontent.com/OpenZeppelin/openzeppelin-contracts/release-v2.5.0/",
        )
    } else if path.starts_with("@openzeppelin/") {
        path.replace(
            "@openzeppelin/",
            "https://raw.githubusercontent.com/OpenZeppelin/openzeppelin-contracts/release-v2.5.0/",
        )
    } else if path.starts_with("https://github.com") && path.contains("/blob/") {
        path.replace("/blob/", "/raw/")
    } else {
        path.to_owned()
    }
}

fn resolve_import_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);

    println!("I: resolving {:?}", path);
    let url = translate_import_url(&path);
    eprintln!("D: resolved {:?}", url);

    let client = reqwest::blocking::Client::builder()
        .timeout(None)
        // .proxy(reqwest::Proxy::https("http://127.0.0.1:8001")?)
        .build()
        .unwrap();
    let source_code = client.get(&url).send().unwrap().text().unwrap();

    // println!("source => {:?}", source);
    retval.set(v8::String::new(scope, &source_code).unwrap().into());
}

#[must_use]
struct SetupGuard {}

impl Drop for SetupGuard {
    fn drop(&mut self) {
        /*
        unsafe {
            v8::V8::dispose();
        }
        v8::V8::shutdown_platform();
        */
    }
}

fn setup() -> SetupGuard {
    let mut g = INIT_LOCK.lock().unwrap();
    *g += 1;
    if *g == 1 {
        v8::V8::initialize_platform(v8::new_default_platform().unwrap());
        v8::V8::initialize();
    }
    SetupGuard {}
}

/// Soljson compiler.
pub struct Compiler {
    _guard: SetupGuard,
    code: String,
}

impl<'a> Compiler {
    pub fn new() -> Result<Compiler, Box<dyn Error>> {
        let config_dir = determine_config_directory();
        let soljson = config_dir.join(SOLJSON_FILENAME);
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        if soljson.exists() {
            Ok(Compiler {
                _guard: setup(),
                code: fs::read_to_string(&soljson)?,
            })
        } else {
            eprintln!("I: downloading compiler...");
            let client = reqwest::blocking::Client::builder()
                .timeout(None)
                // .proxy(reqwest::Proxy::https("http://127.0.0.1:8001")?)
                .build()?;
            let code = client.get(SOLJSON_URL).send()?.text()?;
            let mut file = File::create(&soljson)?;
            file.write_all(code.as_bytes())?;
            eprintln!("I: downloading compiler ok.");
            Ok(Compiler {
                _guard: setup(),
                code: fs::read_to_string(&soljson)?,
            })
        }
    }

    pub fn compile(&self, input: Input) -> Result<Output, Box<dyn Error>> {
        let isolate = &mut v8::Isolate::new(Default::default());
        let scope = &mut v8::HandleScope::new(isolate);

        let global = v8::ObjectTemplate::new(scope);
        global.set(
            v8::String::new(scope, "log").unwrap().into(),
            v8::FunctionTemplate::new(scope, debug_callback).into(),
        );
        global.set(
            v8::String::new(scope, "resolveImport").unwrap().into(),
            v8::FunctionTemplate::new(scope, resolve_import_callback).into(),
        );

        let context = v8::Context::new_from_template(scope, global);
        let scope = &mut v8::ContextScope::new(scope, context);

        eval(scope, &self.code).unwrap();
        eval(scope, include_str!("wrapper.js")).unwrap();

        // println!("=> {}", serde_json::to_string_pretty(&input)?);
        let result = eval(
            scope,
            &format!(
                "compile(JSON.stringify({}), importCallback)",
                serde_json::to_string(&input)?
            ),
        )
        .unwrap();
        let result = result.to_string(scope).unwrap();
        let result = result.to_rust_string_lossy(scope);
        let output: Output = serde_json::from_str(&result)?;

        if output.has_errors() {
            output.format_errors();
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                output.error_message(),
            )));
        }

        Ok(output)
    }
}

fn eval<'s>(scope: &mut v8::HandleScope<'s>, code: &str) -> Option<v8::Local<'s, v8::Value>> {
    let scope = &mut v8::EscapableHandleScope::new(scope);
    let source = v8::String::new(scope, code).unwrap();
    let script = v8::Script::compile(scope, source, None).unwrap();
    let r = script.run(scope);
    r.map(|v| scope.escape(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler() {
        let code = r#"
        contract Store {
            uint256 internal value;

            function reset() external {
                value = 0;
            }

            function setValue(uint256 v) external {
                value = v;
            }
        }
        "#;
        let input = Input::new().optimizer(0).source("Store.sol", code.into());
        let output = Compiler::new().unwrap().compile(input).unwrap();
        println!("=> {:?}", output.bytecode_of("Store").unwrap());
    }

    #[test]
    fn test_compiler_error() {
        let code = r#"
        contract Store {
            uint256 internal value;

            function reset() payable {
                value = 0;
            }

            function setValue(uint256 v) external {
                value = v;
            }
        }
        "#;
        let input = Input::new().optimizer(0).source("Store.sol", code.into());
        let result = Compiler::new().unwrap().compile(input);
        assert!(result.is_err());
        println!("=> {:?}", result);
    }
}
