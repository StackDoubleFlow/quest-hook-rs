use std::ffi::CStr;
use std::fmt;
use std::ops::{Deref, DerefMut};

use crate::{raw, Il2CppObject, WrapRaw};

/// An il2cpp object
#[repr(transparent)]
pub struct Il2CppException(raw::Il2CppException);

impl Il2CppException {
    pub fn format(&self, buf: &mut [u8]) {
        unsafe { raw::format_exception(self.raw(), buf.as_mut_ptr() as _, buf.len() as _) }
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

impl fmt::Debug for Il2CppException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0; 4096];
        self.format(buf.as_mut());
        let message = CStr::from_bytes_with_nul(buf.as_ref())
            .unwrap()
            .to_string_lossy();

        f.debug_struct("Il2CppException")
            .field("class", self.class())
            .field("message", &message)
            .finish()
    }
}
