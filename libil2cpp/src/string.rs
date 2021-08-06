use std::convert::{Infallible, TryFrom};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::string::FromUtf16Error;

use crate::{raw, Il2CppObject, WrapRaw};

/// An il2cpp string
#[repr(transparent)]
pub struct Il2CppString(raw::Il2CppString);

impl Il2CppString {
    /// Creates a new string from a Rust string
    pub fn new(s: impl AsRef<str>) -> &'static mut Self {
        let b = s.as_ref().as_bytes();
        let s = unsafe { raw::string_new_len(b.as_ptr() as _, b.len() as _) };
        unsafe { Self::wrap_mut(s) }
    }

    /// Converts the string to a Rust string, returning an error if its utf-16
    /// data is invalid
    pub fn to_string(&self) -> Result<String, FromUtf16Error> {
        String::from_utf16(self.as_utf16())
    }

    /// Converts the string to a Rust string, replacing any of its invalid data
    /// with the [replacement character
    /// (`U+FFFD`)](core::char::REPLACEMENT_CHARACTER)
    pub fn to_string_lossy(&self) -> String {
        String::from_utf16_lossy(self.as_utf16())
    }

    /// Returns the string's utf-16 data as a slice
    pub fn as_utf16(&self) -> &[u16] {
        let raw = self.raw();
        unsafe { raw.chars.as_slice(raw.length as _) }
    }

    /// Returns the string's utf-16 data as a mutable slice
    pub fn as_utf16_mut(&mut self) -> &mut [u16] {
        let raw = unsafe { self.raw_mut() };
        unsafe { raw.chars.as_mut_slice(raw.length as _) }
    }
}

unsafe impl WrapRaw for Il2CppString {
    type Raw = raw::Il2CppString;
}

impl Deref for Il2CppString {
    type Target = Il2CppObject;

    fn deref(&self) -> &Self::Target {
        unsafe { Il2CppObject::wrap(&self.raw().object) }
    }
}

impl DerefMut for Il2CppString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { Il2CppObject::wrap_mut(&mut self.raw_mut().object) }
    }
}

impl AsRef<[u16]> for Il2CppString {
    fn as_ref(&self) -> &[u16] {
        self.as_utf16()
    }
}

impl AsMut<[u16]> for Il2CppString {
    fn as_mut(&mut self) -> &mut [u16] {
        self.as_utf16_mut()
    }
}

impl<T> From<T> for &'static mut Il2CppString
where
    T: AsRef<str>,
{
    fn from(s: T) -> Self {
        Il2CppString::new(s)
    }
}

impl FromStr for &'static mut Il2CppString {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Il2CppString::new(s))
    }
}

impl TryFrom<&Il2CppString> for String {
    type Error = FromUtf16Error;

    fn try_from(value: &Il2CppString) -> Result<Self, Self::Error> {
        value.to_string()
    }
}

impl<T> PartialEq<T> for Il2CppString
where
    T: AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        self.to_string().map_or(false, |s| s == other.as_ref())
    }
}

impl fmt::Debug for Il2CppString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.to_string_lossy(), f)
    }
}

impl fmt::Display for Il2CppString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.to_string_lossy(), f)
    }
}
