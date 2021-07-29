//! Embedding Node.js in Rust.
//!
//! - Provide a global thread-safe Node.js channel.
//! - Interact with the Node.js runtime via [Neon](https://neon-bindings.com) API.
//! - Link with [prebuilt Node.js binaries](https://github.com/patr0nus/libnode/) to save compile time.
//! - Native modules are supported.
//!
//! ## Guide
//!
//! 1. Copy this repo's ".cargo" folder to your cargo project to enable flags required to link Node.js properly.
//! 2. `let channel = nodejs::channel()` to get the global Node.js channel.
//! 3. Call `channel.send` to run tasks in the Node.js event queue.
//! 4. Inside the task, use `nodejs::neon` for interoperability between Node.js and Rust. [Neon documentation](https://docs.rs/neon/0.9.0/neon/index.html)
//!
//! ## Example
//! ```rust
//! use nodejs::neon::{context::Context, reflect::eval, types::JsNumber};
//!
//! fn main() {
//!     let (tx, rx) = std::sync::mpsc::sync_channel::<i64>(0);
//!     let channel = nodejs::channel();
//!     channel.send(move |mut cx| {
//!         let script = cx.string("require('os').freemem()");
//!         let free_mem = eval(&mut cx, script)?;
//!         let free_mem = free_mem.downcast_or_throw::<JsNumber, _>(&mut cx)?;
//!         tx.send(free_mem.value(&mut cx) as i64).unwrap();
//!         Ok(())
//!     });
//!     let free_mem = rx.recv().unwrap();
//!     println!("Free system memory: {}", free_mem);
//! }
//! ```

mod sys;

pub use neon;
use neon::event::Channel;
use once_cell::sync::Lazy;
use std::env::current_exe;
use std::ffi::CString;
use std::ops::Deref;
use std::os::raw::c_char;
use std::ptr::null_mut;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Mutex;

static CHANNEL_TX_RX: Lazy<(SyncSender<Channel>, Mutex<Receiver<Channel>>)> = Lazy::new(|| {
    let (tx, rx) = sync_channel(0);
    (tx, Mutex::new(rx))
});

#[allow(clippy::unnecessary_wraps)]
mod linked_binding {
    use super::CHANNEL_TX_RX;
    use neon::context::Context;
    use std::sync::Once;

    neon::register_module!(|mut cx| {
        static ONCE: Once = Once::new();
        ONCE.call_once(|| CHANNEL_TX_RX.0.send(cx.channel()).unwrap());
        Ok(())
    });
}

static CHANNEL: Lazy<Channel> = Lazy::new(|| {
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
    let channel = CHANNEL_TX_RX.1.lock().unwrap().recv().unwrap();
    channel
});

/// Ensure the Node.js runtime is running and return the channel.
pub fn channel() -> &'static Channel {
    CHANNEL.deref()
}

#[cfg(test)]
mod tests {
    use super::*;
    use neon::{
        context::Context,
        reflect::eval,
        types::{JsNumber, JsString},
    };

    #[test]
    fn test_with_env() -> anyhow::Result<()> {
        let channel = channel();
        let (tx, rx) = std::sync::mpsc::sync_channel::<i64>(0);
        channel.try_send(move |mut cx| {
            let script = cx.string("6*7");
            let script_result = eval(&mut cx, script)?;
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
        let channel = channel();
        let (tx, rx) = std::sync::mpsc::sync_channel::<String>(0);
        channel.try_send(move |mut cx| {
            let script = cx.string("new URL('http://中文').hostname");
            let hostname = eval(&mut cx, script)?;

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
