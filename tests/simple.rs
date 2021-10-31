use nodejs::neon::{context::Context, types::JsNumber};

#[chazi::test(check_reach)]
fn test_simple() {
    let mut answer = 0;
    unsafe {
        nodejs::run(|mut cx| {
            let script = cx.string("40+2");
            let forty_two = neon::reflect::eval(&mut cx, script)?;
            answer = forty_two
                .downcast_or_throw::<JsNumber, _>(&mut cx)?
                .value(&mut cx) as _;
            Ok(())
        });
    }
    assert_eq!(answer, 42);
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
