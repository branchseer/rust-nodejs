mod common;

use nodejs::neon::{
    context::Context,
    reflect::eval,
    types::{JsFunction, JsNumber, JsString},
};
use std::path::PathBuf;
use fs_extra::dir::{CopyOptions, copy};

#[test]
fn test_require_napi_module() -> anyhow::Result<()> {
    let test_tmpdir = env!("CARGO_TARGET_TMPDIR");
    let napi_module_src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("napi_module");

    let napi_module_installed_dir = PathBuf::from(test_tmpdir)
        .join("napi_module").to_str().unwrap().to_string();

    fs_extra::dir::copy(napi_module_src_dir, &napi_module_installed_dir, &CopyOptions {
        copy_inside: true,
        overwrite: true,
        ..Default::default()
    }).unwrap();
    let mut npm_install_cmd = if cfg!(target_os = "windows") {
        let mut cmd = std::process::Command::new("cmd");
        cmd.arg("/c").arg("npm");
        cmd
    }
    else {
        std::process::Command::new("npm")
    };
    npm_install_cmd
        .current_dir(&napi_module_installed_dir)
        .arg("install");
    if cfg!(target_arch = "x86") {
        npm_install_cmd.arg("--target_arch=ia32");
    }
    let npm_install_status = npm_install_cmd.status()?;
    assert!(npm_install_status.success());

    let add_result = common::sync_node(move |mut cx| {
        let module_path = cx.string(napi_module_installed_dir);
        let js_fn_script = cx.string(
            "module_path => { \
            const m = require(module_path);\
            return m.add(40, 2);\
        }",
        );
        let js_fn = eval(&mut cx, js_fn_script)?;
        let js_fn = js_fn.downcast_or_throw::<JsFunction, _>(&mut cx)?;
        let js_undefined = cx.undefined();
        let js_fn_result = js_fn.call(&mut cx, js_undefined, vec![module_path])?;
        let js_fn_result = js_fn_result.downcast_or_throw::<JsNumber, _>(&mut cx)?;
        Ok(js_fn_result.value(&mut cx) as i64)
    })
    .unwrap();

    assert_eq!(add_result, 42);

    Ok(())
}
