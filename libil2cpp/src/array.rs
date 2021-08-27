use std::marker::PhantomData;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::{fmt, ptr, slice};

use crate::{raw, Il2CppClass, Il2CppObject, Type, WrapRaw};

/// An il2cpp array
#[repr(transparent)]
pub struct Il2CppArray<T: Type>(raw::Il2CppArray, PhantomData<[T]>);

impl<T: Type> Il2CppArray<T> {
    /// Creates an array from a [`Vec`]
    // TODO: Remove this when clippy is fixed
    #[allow(clippy::needless_lifetimes)]
    pub fn new<'a>(items: Vec<T::Held<'a>>) -> &'a mut Self {
        Self::from_iterator(items.into_iter())
    }

    /// Creates an array from an iterator with a known size
    pub fn from_iterator<'a>(items: impl ExactSizeIterator<Item = T::Held<'a>>) -> &'a mut Self {
        let len = items.len();
        let arr = unsafe { raw::array_new(T::class().raw(), len) }.unwrap();
        let data_ptr =
            ((arr as *mut _ as isize) + (raw::kIl2CppSizeOfArray as isize)) as *mut T::Held<'a>;
        for (i, elem) in items.into_iter().enumerate() {
            unsafe {
                let ptr = data_ptr.add(i);
                ptr::write_unaligned(ptr, elem);
            }
        }
        unsafe { Self::wrap_mut(arr) }
    }

    /// Slice of values in the array
    pub fn as_slice(&self) -> &[T::Held<'_>] {
        let ptr = ((self as *const _ as isize) + (raw::kIl2CppSizeOfArray as isize))
            as *const T::Held<'_>;
        let len = self.len();
        unsafe { slice::from_raw_parts(ptr, len) }
    }

    /// Mutable slice of values in the array
    pub fn as_mut_slice(&mut self) -> &mut [T::Held<'_>] {
        let ptr =
            ((self as *mut _ as isize) + (raw::kIl2CppSizeOfArray as isize)) as *mut T::Held<'_>;
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

unsafe impl<T: Type> WrapRaw for Il2CppArray<T> {
    type Raw = raw::Il2CppArray;
}

unsafe impl<T: Type> Type for Il2CppArray<T> {
    type Held<'a> = Option<&'a mut Self>;

    const NAMESPACE: &'static str = "System";
    const CLASS_NAME: &'static str = "Array";

    fn class() -> &'static Il2CppClass {
        let class = unsafe { raw::array_class_get(T::class().raw(), 0, false) };
        unsafe { Il2CppClass::wrap(class) }
    }

    fn matches_reference_argument(ty: &crate::Il2CppType) -> bool {
        ty.class().is_assignable_from(Self::class())
    }

    fn matches_value_argument(_: &crate::Il2CppType) -> bool {
        false
    }

    fn matches_reference_parameter(ty: &crate::Il2CppType) -> bool {
        Self::class().is_assignable_from(ty.class())
    }

    fn matches_value_parameter(_: &crate::Il2CppType) -> bool {
        false
    }
}

impl<T: Type> fmt::Debug for Il2CppArray<T>
where
    for<'a> T::Held<'a>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Il2CppArray")
            .field(&self.as_slice())
            .finish()
    }
}

impl<T: Type> Deref for Il2CppArray<T> {
    type Target = Il2CppObject;

    fn deref(&self) -> &Self::Target {
        unsafe { Il2CppObject::wrap(&self.raw().obj) }
    }
}

impl<T: Type> DerefMut for Il2CppArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { Il2CppObject::wrap_mut(&mut self.raw_mut().obj) }
    }
}
