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

    /// Iterator over the inner exceptions, starting with the exception itself
    pub fn trace(&self) -> Trace<'_> {
        Trace {
            current: Some(self),
        }
    }

    /// Exception source
    pub fn source(&self) -> Option<&Il2CppString> {
        unsafe { Il2CppString::wrap_ptr(self.raw().source) }
    }

    /// Throws the exception
    ///
    /// # Safety
    /// This is implemented as a C++ throw, which is UB when called from Rust.
    /// Therefore this method is UB, and only provided just in case ™️. (in
    /// simpler terms, this method is never safe)
    pub unsafe fn throw(&self) -> ! {
        raw::raise_exception(self.raw())
    }
}

/// Iterator over inner exceptions
#[derive(Debug)]
pub struct Trace<'a> {
    current: Option<&'a Il2CppException>,
}

unsafe impl WrapRaw for Il2CppException {
    type Raw = raw::Il2CppException;
}

impl<'a> Iterator for Trace<'a> {
    type Item = &'a Il2CppException;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(e) => {
                self.current = e.inner_exception();
                Some(e)
            }
            None => None,
        }
    }
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
            .field("source", &self.source())
            .finish()
    }
}

impl std::error::Error for Il2CppException {}
impl std::error::Error for &mut Il2CppException {}
