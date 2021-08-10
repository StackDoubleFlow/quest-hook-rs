use std::os::raw::c_int;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};

extern "C" {
    fn registerInlineHook(target_addr: u32, new_addr: u32, proto_addr: *mut *mut u32) -> c_int;
    fn inlineHook(target_addr: u32) -> c_int;
}

/// A function hook specific to `ARMv7` Android
#[derive(Debug)]
pub struct Hook {
    original: AtomicPtr<u32>,
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
        let target = target as u32;
        let hook = hook as u32;
        let mut original: *mut u32 = null_mut();

        if registerInlineHook(target, hook, &mut original) != 0 || inlineHook(target) != 0 {
            return false;
        }

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
