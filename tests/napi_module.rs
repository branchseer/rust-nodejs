mod common;

use nodejs::neon::{
    context::Context,
    reflect::eval,
    types::{JsFunction, JsNumber, JsString},
};
use std::path::PathBuf;

#[test]
fn test_require_napi_module() -> anyhow::Result<()> {
    let test_tmpdir = env!("CARGO_TARGET_TMPDIR");
    let napi_module_src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("napi_module");
    let npm_install_cmd = std::process::Command::new("npm")
        .current_dir(test_tmpdir)
        .arg("install")
        .arg(napi_module_src_dir);
    if cfg!(target_arch = "x86") {
        npm_install_cmd.arg("--target_arch=ia32")
    }
    let npm_install_status = npm_install_cmd.status()?;
    assert!(npm_install_status.success());

    let require_from = PathBuf::from(test_tmpdir)
        .join("index.js")
        .to_str()
        .unwrap()
        .to_string();

    let add_result = common::sync_node(move |mut cx| {
        let require_from = cx.string(require_from);
        let js_fn_script = cx.string(
            "require_from => { \
            const require_fn = module.createRequire(require_from);\
            const m = require_fn('napi_module');\
            return m.add(40, 2);\
        }",
        );
        let js_fn = eval(&mut cx, js_fn_script)?;
        let js_fn = js_fn.downcast_or_throw::<JsFunction, _>(&mut cx)?;
        let js_undefined = cx.undefined();
        let js_fn_result = js_fn.call(&mut cx, js_undefined, vec![require_from])?;
        let js_fn_result = js_fn_result.downcast_or_throw::<JsNumber, _>(&mut cx)?;
        Ok(js_fn_result.value(&mut cx) as i64)
    })
    .unwrap();

    assert_eq!(add_result, 42);

    Ok(())
}
