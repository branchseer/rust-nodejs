use std::os::raw::c_char;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
fn res_path() -> PathBuf {
    use objc::{runtime::Object, *};
    use std::ffi::{CStr, CString};
    let resource_path = rc::autoreleasepool(|| unsafe {
        let ns_bundle_cls = class!(NSBundle);
        let main_bundle: *mut Object = msg_send![ns_bundle_cls, mainBundle];
        let resource_path: *mut Object = msg_send![main_bundle, resourcePath];
        let resource_path: *const c_char = msg_send![resource_path, UTF8String];
        CStr::from_ptr(resource_path).to_bytes().to_vec()
    });
    panic!("dwq")
}

pub unsafe fn main() {
    let _ = res_path();
}
