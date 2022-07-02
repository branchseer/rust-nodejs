use std::os::raw::{c_char, c_void};
use std::path::{Path, PathBuf};
use path_absolutize::Absolutize;

#[cfg(target_os = "macos")]
fn platform_res_path() -> Option<PathBuf> {
    use objc::{runtime::Object, *};
    use std::ffi::CStr;
    use std::os::unix::ffi::OsStringExt;
    use std::ffi::OsString;

    let resource_path = rc::autoreleasepool(|| unsafe {
        let ns_bundle_cls = class!(NSBundle);
        let main_bundle: *mut Object = msg_send![ns_bundle_cls, mainBundle];
        let resource_path: *mut Object = msg_send![main_bundle, resourcePath];
        let resource_path: *const c_char = msg_send![resource_path, UTF8String];
        CStr::from_ptr(resource_path).to_bytes().to_vec()
    });
    let resource_path = PathBuf::from(OsString::from_vec(resource_path));
    Some(resource_path)
}

#[cfg(not(target_os = "macos"))]
fn platform_res_path() -> Option<PathBuf> {
    None
}


unsafe extern "C" fn node_start(napi_reg_func: *mut c_void) -> i32 {
    nodejs::run_raw(napi_reg_func)
}

pub unsafe fn main() -> i32 {
    let exec_dir =  std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let res_path = platform_res_path().unwrap_or_else(|| exec_dir.clone());

    let embedder_path_file = res_path.join("nodejs_embedder");
    let dylib_path = std::fs::read_to_string(embedder_path_file).unwrap();
    let dylib_path = Path::new(dylib_path.trim()).absolutize_from(&exec_dir).unwrap();

    let embedder_lib = libloading::Library::new(dylib_path.as_ref()).unwrap();
    let embedder_main: libloading::Symbol<unsafe extern "C" fn(*mut c_void) -> i32> = embedder_lib.get(b"node_embedder_main").unwrap();
    embedder_main(node_start as _)
}
