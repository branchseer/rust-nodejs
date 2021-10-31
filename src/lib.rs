#![doc = include_str!("../README.md")]

mod sys;

pub use neon;
use neon::context::ModuleContext;
use neon::result::NeonResult;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr::null_mut;

pub unsafe fn run<F: for<'a> FnOnce(ModuleContext<'a>) -> NeonResult<()>>(f: F) -> i32 {
    static mut MODULE_INIT_FN: *mut std::ffi::c_void = null_mut(); // *mut Option<F>

    let mut module_init_fn = Some(f);
    MODULE_INIT_FN = (&mut module_init_fn) as *mut Option<F> as _;

    unsafe extern "C" fn napi_reg_func<F: for<'a> FnOnce(ModuleContext<'a>) -> NeonResult<()>>(
        env: neon::macro_internal::runtime::raw::Env,
        m: neon::macro_internal::runtime::raw::Local,
    ) -> neon::macro_internal::runtime::raw::Local {
        neon::macro_internal::initialize_module(env, std::mem::transmute(m), |ctx| {
            let module_init_fn = (MODULE_INIT_FN as *mut Option<F>).as_mut().unwrap();
            let module_init_fn = module_init_fn.take().unwrap();
            MODULE_INIT_FN = null_mut();
            module_init_fn(ctx)
        });
        m
    }

    let args: Vec<String> = std::env::args().collect();
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
