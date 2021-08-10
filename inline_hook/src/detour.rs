use std::lazy::SyncOnceCell;

use detour::RawDetour;

/// A function hook that works across most platforms
#[derive(Debug)]
pub struct Hook {
    detour: SyncOnceCell<RawDetour>,
}

impl Hook {
    /// Creates a new, unitialized hook
    pub const fn new() -> Self {
        Self {
            detour: SyncOnceCell::new(),
        }
    }

    /// Installes the hook by redirecting `target` to `hook`, returning true on
    /// success
    ///
    /// # Safety
    /// `target` and `hook` must have the same signature and calling convention
    pub unsafe fn install(&self, target: *const (), hook: *const ()) -> bool {
        match RawDetour::new(target, hook) {
            Ok(detour) if detour.enable().is_ok() => {
                self.detour.set(detour).ok();
                true
            }
            _ => false,
        }
    }

    /// Whether the hook is installed
    pub fn is_installed(&self) -> bool {
        self.detour.get().is_some()
    }

    /// Returns the address of a trampoline function to the original target, if
    /// installed
    pub fn original(&self) -> Option<*const ()> {
        self.detour.get().map(|d| d.trampoline() as *const ())
    }
}
