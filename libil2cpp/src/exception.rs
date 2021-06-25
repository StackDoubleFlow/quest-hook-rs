use std::fmt;
use std::ops::{Deref, DerefMut};

use crate::{raw, Il2CppObject, Il2CppString, WrapRaw};

/// An il2cpp exception
#[repr(transparent)]
pub struct Il2CppException(raw::Il2CppException);

impl Il2CppException {
    /// Exception message
    pub fn message(&self) -> Option<&Il2CppString> {
        unsafe { Il2CppString::wrap_ptr(self.raw().message) }
    }

    /// Inner exception
    pub fn inner_exception(&self) -> Option<&Self> {
        unsafe { Self::wrap_ptr(self.raw().inner_ex) }
    }

    /// Stack trace
    pub fn stack_trace(&self) -> Option<&Il2CppString> {
        unsafe { Il2CppString::wrap_ptr(self.raw().stack_trace) }
    }

    /// Exception source
    pub fn source(&self) -> Option<&Il2CppString> {
        unsafe { Il2CppString::wrap_ptr(self.raw().source) }
    }
}

unsafe impl WrapRaw for Il2CppException {
    type Raw = raw::Il2CppException;
}

impl Deref for Il2CppException {
    type Target = Il2CppObject;

    fn deref(&self) -> &Self::Target {
        unsafe { Il2CppObject::wrap(&self.raw().object) }
    }
}

impl DerefMut for Il2CppException {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { Il2CppObject::wrap_mut(&mut self.raw_mut().object) }
    }
}

impl fmt::Display for Il2CppException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.message() {
            Some(m) => write!(f, "{}: {}", self.class(), m),
            None => fmt::Display::fmt(self.class(), f),
        }
    }
}

impl fmt::Debug for Il2CppException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Il2CppException")
            .field("class", self.class())
            .field("message", &self.message())
            .field("stack_trace", &self.stack_trace())
            .field("source", &self.source())
            .finish()
    }
}
