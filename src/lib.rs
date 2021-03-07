mod sys;

pub use neon;
use neon::event::EventQueue;
use once_cell::sync::Lazy;
use std::env::current_exe;
use std::ffi::CString;
use std::ops::Deref;
use std::os::raw::c_char;
use std::ptr::null_mut;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Mutex;

static QUEUE_CHANNEL: Lazy<(SyncSender<EventQueue>, Mutex<Receiver<EventQueue>>)> =
    Lazy::new(|| {
        let (tx, rx) = sync_channel(0);
        (tx, Mutex::new(rx))
    });

#[allow(clippy::unnecessary_wraps)]
mod linked_binding {
    use super::QUEUE_CHANNEL;
    use neon::context::Context;
    use std::sync::Once;

    neon::register_module!(|mut cx| {
        static ONCE: Once = Once::new();
        ONCE.call_once(|| QUEUE_CHANNEL.0.send(cx.queue()).unwrap());
        Ok(())
    });
}

static QUEUE: Lazy<EventQueue> = Lazy::new(|| {
    std::thread::spawn(|| {
        const LINKED_BINDING_NAME: &str = "__rust_init";

        unsafe extern "C" fn register_linked_binding(
            raw_env: napi_sys::napi_env,
            raw_exports: napi_sys::napi_value,
        ) -> napi_sys::napi_value {
            linked_binding::napi_register_module_v1(raw_env as _, raw_exports as _) as _
        }
        let mut nm = napi_sys::napi_module {
            nm_version: -1,
            nm_flags: 0,
            nm_filename: concat!(file!(), '\0').as_ptr() as *const c_char,
            nm_register_func: Some(register_linked_binding),
            nm_modname: CString::new(LINKED_BINDING_NAME).unwrap().into_raw(),
            nm_priv: null_mut(),
            reserved: [null_mut(); 4],
        };
        unsafe { napi_sys::napi_module_register(&mut nm) };

        let mut argv0 = current_exe()
            .ok()
            .map(|p| p.to_str().map(str::to_string))
            .flatten()
            .unwrap_or_else(|| "node".to_string())
            + "\0";
        let mut init_code = format!("process._linkedBinding('{}')\0", LINKED_BINDING_NAME);
        let mut option_e = "-e\0".to_string();
        let args = &mut [
            argv0.as_mut_ptr() as *mut c_char,
            option_e.as_mut_ptr() as *mut c_char,
            init_code.as_mut_ptr() as *mut c_char,
        ][..];
        unsafe {
            sys::node_start(args.len() as i32, args.as_mut_ptr());
        }
        panic!("Node.js runtime closed expectedly")
    });
    let queue = QUEUE_CHANNEL.1.lock().unwrap().recv().unwrap();
    queue
});

pub fn event_queue() -> &'static EventQueue {
    QUEUE.deref()
}

#[cfg(test)]
mod tests {
    use super::*;
    use neon::context::Context;
    use neon::types::{JsNumber, JsString};

    #[test]
    fn test_with_env() -> anyhow::Result<()> {
        let queue = event_queue();
        let (tx, rx) = std::sync::mpsc::sync_channel::<i64>(0);
        queue.try_send(move |mut cx| {
            let script_result = cx.run_script("6*7")?;
            let script_result = script_result.downcast_or_throw::<JsNumber, _>(&mut cx)?;
            tx.send(script_result.value(&mut cx) as i64).unwrap();
            Ok(())
        })?;
        let script_result = rx.recv()?;
        assert_eq!(script_result, 42);
        Ok(())
    }

    #[test]
    fn test_intl() -> anyhow::Result<()> {
        let queue = event_queue();
        let (tx, rx) = std::sync::mpsc::sync_channel::<String>(0);
        queue.try_send(move |mut cx| {
            let hostname = cx.run_script("new URL('http://中文').hostname")?;
            let hostname = hostname.downcast_or_throw::<JsString, _>(&mut cx)?;
            tx.send(hostname.value(&mut cx)).unwrap();
            Ok(())
        })?;
        let hostname = rx.recv()?;
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
}
