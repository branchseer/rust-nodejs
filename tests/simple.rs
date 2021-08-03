mod common;

use nodejs::neon::{
    context::Context,
    reflect::eval,
    types::{JsNumber, JsString},
};

#[test]
fn test_simple_expr() {
    let script_result = common::sync_node(move |mut cx| {
        let script = cx.string("6*7");
        let script_result = eval(&mut cx, script)?;
        let script_result = script_result.downcast_or_throw::<JsNumber, _>(&mut cx)?;
        Ok(script_result.value(&mut cx) as i64)
    })
    .unwrap();
    assert_eq!(script_result, 42);
}

#[test]
fn test_intl() -> anyhow::Result<()> {
    let hostname = common::sync_node(move |mut cx| {
        let script = cx.string("new URL('http://中文').hostname");
        let hostname = eval(&mut cx, script)?;
        let hostname = hostname.downcast_or_throw::<JsString, _>(&mut cx)?;
        Ok(hostname.value(&mut cx))
    })
    .unwrap();
    assert_eq!(
        hostname,
        if cfg!(feature = "no-intl") {
            "中文"
        } else {
            "xn--fiq228c"
        }
    );
    Ok(())
}
