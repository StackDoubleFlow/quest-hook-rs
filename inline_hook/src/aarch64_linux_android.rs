use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};

extern "C" {
    fn A64HookFunction(symbol: *const c_void, replace: *const c_void, result: *mut *mut c_void);
}

/// A function hook specific to ARMv8 Android
#[derive(Debug)]
pub struct Hook {
    original: AtomicPtr<c_void>,
}

impl Hook {
    /// Creates a new, unitialized hook
    pub const fn new() -> Self {
        Self {
            original: AtomicPtr::new(null_mut()),
        }
    }

    /// Installes the hook by redirecting `target` to `hook`, returning true on
    /// success
    ///
    /// # Safety
    /// `target` and `hook` must have the same signature and calling convention
    pub unsafe fn install(&self, target: *const (), hook: *const ()) -> bool {
        let target = target as *const c_void;
        let hook = hook as *const c_void;
        let mut original: *mut c_void = null_mut();

        A64HookFunction(target, hook, &mut original);

        self.original.store(original, Ordering::SeqCst);
        true
    }

    /// Whether the hook is installed
    pub fn is_installed(&self) -> bool {
        !self.original.load(Ordering::SeqCst).is_null()
    }

    /// Returns the address of a trampoline function to the original target, if
    /// installed
    pub fn original(&self) -> Option<*const ()> {
        match self.original.load(Ordering::SeqCst) {
            null if null.is_null() => None,
            original => Some(original as *const ()),
        }
    }
}
