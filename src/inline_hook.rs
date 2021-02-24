use std::ffi::c_void;

extern "C" {
    pub fn A64HookFunction(symbol: *mut c_void, replace: *mut c_void, result: *mut *mut c_void);
}
