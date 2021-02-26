pub use quest_hook_proc_macros::hook;
use std::ffi::c_void;

extern "C" {
    pub fn A64HookFunction(symbol: *mut c_void, replace: *mut c_void, result: *mut *mut c_void);
}

trait Hook {
    fn namespace(&self) -> &'static str;
    fn class_name(&self) -> &'static str;
    fn method_name(&self) -> &'static str;
    fn arg_count(&self) -> usize;
}
