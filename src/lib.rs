#![doc = include_str!("../README.md")]

mod sys;

pub use neon;
use neon::context::ModuleContext;
use neon::result::NeonResult;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr::null_mut;
use std::sync::Once;

/// Starts a Node.js instance and immediately run the provided N-API module init function.
/// Blocks until the event loop stops, and returns the exit code.
/// # Safety
/// This function can only be called at most once.
pub unsafe fn run<F: for<'a> FnOnce(ModuleContext<'a>) -> NeonResult<()>>(f: F) -> i32 {
    static mut MODULE_INIT_FN: *mut std::ffi::c_void = null_mut(); // *mut Option<F>

    let mut module_init_fn = Some(f);
    MODULE_INIT_FN = (&mut module_init_fn) as *mut Option<F> as _;

    unsafe extern "C" fn napi_reg_func<F: for<'a> FnOnce(ModuleContext<'a>) -> NeonResult<()>>(
        env: neon::macro_internal::runtime::raw::Env,
        m: neon::macro_internal::runtime::raw::Local,
    ) -> neon::macro_internal::runtime::raw::Local {
        neon::macro_internal::initialize_module(env, std::mem::transmute(m), |ctx| {
            static ONCE: Once = Once::new();
            let mut result = NeonResult::Ok(());
            ONCE.call_once(|| {
                let module_init_fn = (MODULE_INIT_FN as *mut Option<F>).as_mut().unwrap();
                let module_init_fn = module_init_fn.take().unwrap();
                MODULE_INIT_FN = null_mut();
                result = module_init_fn(ctx)
            });
            result
        });
        m
    }

    let args: Vec<CString> = std::env::args()
        .map(|arg| CString::new(arg).unwrap_or_default())
        .collect();
    let mut argc_c = Vec::<*const c_char>::with_capacity(args.len());
    for arg in &args {
        argc_c.push(arg.as_ptr() as *const c_char)
    }

    let result = sys::node_run(sys::node_options_t {
        process_argc: argc_c.len() as c_int,
        process_argv: argc_c.as_ptr(),
        napi_reg_func: napi_reg_func::<F> as _,
    });

    if !result.error.is_null() {
        let result_error_string = CString::from(CStr::from_ptr(result.error));
        libc::free(result.error as _);
        panic!("Node.js failed to start: {:?}", result_error_string);
    }
    result.exit_code as i32
}
