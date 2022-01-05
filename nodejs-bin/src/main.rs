use std::ptr::null_mut;

fn main() {
    unsafe {
        nodejs::run_raw(null_mut());
    }
}
