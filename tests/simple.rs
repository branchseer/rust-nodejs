use nodejs::neon::result::NeonResult;
use nodejs::neon::types::{JsArray, JsString};
use nodejs::neon::{context::Context, types::JsNumber};

#[chazi::test(check_reach)]
fn test_simple() {
    let mut answer = 0;
    let exit_code = unsafe {
        nodejs::run(|mut cx| {
            let script = cx.string("40+2");
            let forty_two = neon::reflect::eval(&mut cx, script)?;
            answer = forty_two
                .downcast_or_throw::<JsNumber, _>(&mut cx)?
                .value(&mut cx) as _;
            Ok(())
        })
    };
    assert_eq!(answer, 42);
    assert_eq!(exit_code, 0);
    chazi::reached::last()
}

#[chazi::test(check_reach)]
fn test_process_exit() {
    let exit_code = unsafe {
        nodejs::run(|mut cx| {
            let script = cx.string("process.exit(40+2)");
            neon::reflect::eval(&mut cx, script)?;
            Ok(())
        })
    };
    assert_eq!(exit_code, 42);
    chazi::reached::last()
}

#[chazi::test(check_reach)]
fn test_argv() {
    let mut args = Vec::<String>::new();
    let exit_code = unsafe {
        nodejs::run(|mut cx| {
            let script = cx.string("[process.argv0, ...process.argv.slice(1)]");
            let process_args = neon::reflect::eval(&mut cx, script)?;
            let process_args = process_args
                .downcast_or_throw::<JsArray, _>(&mut cx)?
                .to_vec(&mut cx)?;
            args = process_args
                .iter()
                .map(|arg| {
                    Ok(arg
                        .downcast_or_throw::<JsString, _>(&mut cx)?
                        .value(&mut cx))
                })
                .collect::<NeonResult<Vec<String>>>()?;
            Ok(())
        })
    };
    assert_eq!(args, std::env::args().collect::<Vec<String>>());
    assert_eq!(exit_code, 0);
    chazi::reached::last()
}

#[chazi::test(check_reach)]
fn test_uncaught_error() {
    let exit_code = unsafe {
        nodejs::run(|mut cx| {
            let script = cx.string("setImmediate(() => throw new Error())");
            neon::reflect::eval(&mut cx, script)?;
            Ok(())
        })
    };
    assert_eq!(exit_code, 1);
    chazi::reached::last()
}
