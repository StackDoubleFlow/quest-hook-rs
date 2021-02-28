//! Raw il2cpp types and functions
//!
//! This module contains raw C types defined in libil2cpp and raw C functions
//! dynamically loaded from libil2cpp.so.

mod functions;
mod types;

pub use functions::*;
pub use types::*;

use std::mem::transmute;

/// Safe wrapper around a raw il2cpp type which can be used in its place
///
/// # Safety
/// The wrapper must have the exact same representation as the underlying raw
/// il2cpp type, which means it has to be `#[repr(transparent)]`.
pub unsafe trait WrapRaw: Sized {
    /// Raw il2cpp type
    type Raw;

    /// Returns a reference to the underlying raw il2cpp type
    fn raw(&self) -> &Self::Raw {
        unsafe { &*(self as *const _ as *const _) }
    }

    /// Returns a mutable reference to the underlying raw il2cpp type
    ///
    /// # Safety
    /// This method is unsafe because it allows mutating the underlying type in
    /// ways that make it invalid. Avoid mutating raw il2cpp types unless you
    /// know exactly what you are doing.
    unsafe fn raw_mut(&mut self) -> &mut Self::Raw {
        &mut *(self as *mut _ as *mut _)
    }

    /// Wraps a reference to the raw il2cpp type
    ///
    /// # Safety
    /// The wrapped type must be in a valid state.
    unsafe fn wrap(raw: &Self::Raw) -> &Self {
        &*(raw as *const _ as *const _)
    }

    /// Wraps a mutable reference to the raw il2cpp type
    ///
    /// # Safety
    /// The wrapped type must be in a valid state.
    unsafe fn wrap_mut(raw: &mut Self::Raw) -> &mut Self {
        &mut *(raw as *mut _ as *mut _)
    }

    /// Wraps a const pointer to the raw il2cpp type
    ///
    /// # Safety
    /// The pointer must not be dangling and must stay valid for the lifetime of
    /// the returned reference if it is not null, and the wrapped type must be
    /// in a valid state.
    unsafe fn wrap_ptr<'a>(ptr: *const Self::Raw) -> Option<&'a Self> {
        transmute(ptr)
    }

    /// Wraps a mut pointer to the raw il2cpp type
    ///
    /// # Safety
    /// The pointer must not be dangling and must stay valid for the lifetime of
    /// the returned mutable reference if it is not null, and the wrapped type
    /// must be in a valid state.
    unsafe fn wrap_ptr_mut<'a>(ptr: *mut Self::Raw) -> Option<&'a mut Self> {
        transmute(ptr)
    }
}
