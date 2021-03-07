use neon::context::Context;
use neon::handle::Handle;
use neon::object::Object;
use neon::types::{JsFunction, JsValue};

pub trait NeonContextExt<'a> {
    fn run(&mut self, code: impl AsRef<str>) -> Result<Handle<'a, JsValue>, Handle<'a, JsValue>>;
}

impl<'a, C: Context<'a>> NeonContextExt<'a> for C {
    fn run(&mut self, code: impl AsRef<str>) -> Result<Handle<'a, JsValue>, Handle<'a, JsValue>> {
        self.try_catch(|cx| {
            let function_constructor = cx.global().get(cx, "Function")?;
            let function_constructor =
                function_constructor.downcast_or_throw::<JsFunction, _>(cx)?;

            let code_string = cx.string(code);

            let function = function_constructor.construct(cx, [code_string].iter().cloned())?;
            let function = function.downcast_or_throw::<JsFunction, _>(cx)?;

            let null = cx.null();
            function.call(cx, null, std::iter::empty::<Handle<'_, JsValue>>())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_queue;
    use neon::types::{JsNumber, JsString};

    #[test]
    fn test_run_script() -> anyhow::Result<()> {
        let queue = event_queue();
        let (tx, rx) = std::sync::mpsc::sync_channel::<i64>(0);
        queue.try_send(move |mut cx| {
            let val = cx.run("return 6*7").ok().unwrap();
            let val = val.downcast_or_throw::<JsNumber, _>(&mut cx)?;
            tx.send(val.value(&mut cx) as i64).unwrap();
            Ok(())
        })?;
        let val = rx.recv()?;
        assert_eq!(val, 42);
        Ok(())
    }

    #[test]
    fn test_run_script_throw() -> anyhow::Result<()> {
        let queue = event_queue();
        let (tx, rx) = std::sync::mpsc::sync_channel::<String>(0);
        queue.try_send(move |mut cx| {
            let err = cx.run("throw 'b1-66er'").err().unwrap();
            let err_string = err.downcast_or_throw::<JsString, _>(&mut cx)?;
            tx.send(err_string.value(&mut cx)).unwrap();
            Ok(())
        })?;
        let val = rx.recv()?;
        assert_eq!(val, "b1-66er");
        Ok(())
    }
}
