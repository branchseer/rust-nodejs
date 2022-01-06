use std::os::raw::c_char;
use std::path::PathBuf;

#[cfg(not(target_os = "macos"))]
fn res_path() -> PathBuf {
    std::env::current_exe().unwrap().parent().unwrap().to_path_buf()
}

#[cfg(target_os = "macos")]
fn res_path() -> PathBuf {
    use objc::{runtime::Object, *};
    use std::ffi::{CStr, CString};
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
    resource_path
}

pub unsafe fn main() {
    let resource_path = res_path();
    let embedder_path_file = resource_path.join("node_embedder");
    println!("{:?}", embedder_path_file)
}
