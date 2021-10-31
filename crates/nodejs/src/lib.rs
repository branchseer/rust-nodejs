#![doc = include_str!("../README.md")]

use std::os::raw::{c_char, c_int};
use neon;

pub fn hello() {
    unsafe extern "C" fn napi_reg_func(
        env: neon::macro_internal::runtime::raw::Env,
        m: neon::macro_internal::runtime::raw::Local,
    ) -> neon::macro_internal::runtime::raw::Local {
        neon::macro_internal::initialize_module(
            env,
            std::mem::transmute(m),
            |ctx| {
                eprintln!("hello");
                Ok(())
            }
        );
        m
    }
    let args: Vec<String> = std::env::args().collect();
    let mut argc_c = Vec::<*const c_char>::with_capacity(args.len());
    for arg in &args {
        argc_c.push(arg.as_ptr() as *const c_char)
    }
    unsafe {
        nodejs_sys::node_run(nodejs_sys::node_options_t {
            process_argc: argc_c.len() as c_int,
            process_argv: argc_c.as_ptr(),
            napi_reg_func: napi_reg_func as _
        });
    }
}
//
// mod sys;
//
// pub use neon;
// use neon::event::Channel;
// use once_cell::sync::Lazy;
// use std::env::current_exe;
// use std::ffi::CString;
// use std::ops::Deref;
// use std::os::raw::c_char;
// use std::ptr::null_mut;
// use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
// use std::sync::Mutex;
//
// static CHANNEL_TX_RX: Lazy<(SyncSender<Channel>, Mutex<Receiver<Channel>>)> = Lazy::new(|| {
//     let (tx, rx) = sync_channel(0);
//     (tx, Mutex::new(rx))
// });
//
// #[allow(clippy::unnecessary_wraps)]
// mod linked_binding {
//     use super::CHANNEL_TX_RX;
//     use neon::context::Context;
//     use std::sync::Once;
//
//     neon::register_module!(|mut cx| {
//         static ONCE: Once = Once::new();
//         ONCE.call_once(|| CHANNEL_TX_RX.0.send(cx.channel()).unwrap());
//         Ok(())
//     });
// }
//
// static CHANNEL: Lazy<Channel> = Lazy::new(|| {
//     std::thread::spawn(|| {
//         const LINKED_BINDING_NAME: &str = "__rust_init";
//
//         unsafe extern "C" fn register_linked_binding(
//             raw_env: napi_sys::napi_env,
//             raw_exports: napi_sys::napi_value,
//         ) -> napi_sys::napi_value {
//             linked_binding::napi_register_module_v1(raw_env as _, raw_exports as _) as _
//         }
//         let mut nm = napi_sys::napi_module {
//             nm_version: -1,
//             nm_flags: 0,
//             nm_filename: concat!(file!(), '\0').as_ptr() as *const c_char,
//             nm_register_func: Some(register_linked_binding),
//             nm_modname: CString::new(LINKED_BINDING_NAME).unwrap().into_raw(),
//             nm_priv: null_mut(),
//             reserved: [null_mut(); 4],
//         };
//         unsafe { napi_sys::napi_module_register(&mut nm) };
//
//         let mut argv0 = current_exe()
//             .ok()
//             .map(|p| p.to_str().map(str::to_string))
//             .flatten()
//             .unwrap_or_else(|| "node".to_string())
//             + "\0";
//         let mut init_code = format!("process._linkedBinding('{}')\0", LINKED_BINDING_NAME);
//         let mut option_e = "-e\0".to_string();
//         let args = &mut [
//             argv0.as_mut_ptr() as *mut c_char,
//             option_e.as_mut_ptr() as *mut c_char,
//             init_code.as_mut_ptr() as *mut c_char,
//         ][..];
//         unsafe {
//             sys::node_start(args.len() as i32, args.as_mut_ptr());
//         }
//         panic!("Node.js runtime closed expectedly")
//     });
//     let channel = CHANNEL_TX_RX.1.lock().unwrap().recv().unwrap();
//     channel
// });
//
// /// Ensure the Node.js runtime is running and return the channel for queuing tasks into the Node.js event loop.
// pub fn channel() -> &'static Channel {
//     CHANNEL.deref()
// }
