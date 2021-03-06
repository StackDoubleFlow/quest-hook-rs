use std::marker::PhantomData;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::{fmt, ptr, slice};

use crate::{raw, Il2CppObject, Type, WrapRaw};

/// An il2cpp array
#[repr(transparent)]
pub struct Il2CppArray<T>(raw::Il2CppArray, PhantomData<[T]>);

impl<T> Il2CppArray<T> {
    /// Creates an array from a slice
    pub fn from_slice(slice: &[T]) -> &Il2CppArray<T>
    where
        T: Clone + Type,
    {
        let len = slice.len();
        let arr = unsafe { raw::array_new(T::class().raw(), len) };
        let data_ptr = ((arr as isize) + (raw::kIl2CppSizeOfArray as isize)) as *mut T;
        for (i, elem) in slice.iter().enumerate() {
            unsafe {
                let ptr = data_ptr.add(i);
                ptr::write_unaligned(ptr, elem.clone());
            }
        }
        unsafe { Il2CppArray::wrap_ptr_mut(arr) }.unwrap()
    }

    /// Slice of values in the array
    pub fn as_slice(&self) -> &[T] {
        let ptr = ((self as *const _ as isize) + (raw::kIl2CppSizeOfArray as isize)) as *const T;
        let len = self.len();
        unsafe { slice::from_raw_parts(ptr, len) }
    }

    /// Mutable slice of values in the array
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let ptr = ((self as *mut _ as isize) + (raw::kIl2CppSizeOfArray as isize)) as *mut T;
        let len = self.len();
        unsafe { slice::from_raw_parts_mut(ptr, len) }
    }

    /// Length of the array
    pub fn len(&self) -> usize {
        let raw = self.raw();
        let bounds: Option<&raw::Il2CppArrayBounds> = unsafe { transmute(raw.bounds) };
        match bounds {
            Some(bounds) => bounds.length,
            None => raw.max_length,
        }
    }

    /// Whether the array is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

unsafe impl<T> WrapRaw for Il2CppArray<T> {
    type Raw = raw::Il2CppArray;
}

impl<T: fmt::Debug> fmt::Debug for Il2CppArray<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Il2CppArray").field(&self.as_ref()).finish()
    }
}

impl<T> Deref for Il2CppArray<T> {
    type Target = Il2CppObject;

    fn deref(&self) -> &Self::Target {
        unsafe { Il2CppObject::wrap(&self.raw().obj) }
    }
}

impl<T> DerefMut for Il2CppArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { Il2CppObject::wrap_mut(&mut self.raw_mut().obj) }
    }
}

impl<T> AsRef<[T]> for Il2CppArray<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> AsMut<[T]> for Il2CppArray<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}
