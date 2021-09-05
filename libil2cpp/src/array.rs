use std::marker::PhantomData;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::{fmt, ptr, slice};

use crate::{raw, Il2CppClass, Il2CppObject, Type, WrapRaw};

/// An il2cpp array
#[repr(transparent)]
pub struct Il2CppArray<T: Type>(raw::Il2CppArray, PhantomData<[T]>);

impl<T: Type> Il2CppArray<T> {
    /// Creates an array from an iterator
    pub fn new<'a, I>(items: I) -> &'a mut Self
    where
        I: IntoIterator<Item = T::Held<'a>>,
        I::IntoIter: ExactSizeIterator<Item = T::Held<'a>>,
    {
        let items = items.into_iter();
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
    type HeldRaw = *mut Self;

    const NAMESPACE: &'static str = "System";
    const CLASS_NAME: &'static str = "Array";

    fn class() -> &'static Il2CppClass {
        let class = unsafe { raw::array_class_get(T::class().raw(), 1) };
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

#[cfg(feature = "serde")]
mod serde {
    use super::{fmt, ptr, raw, Il2CppArray, PhantomData, Type, WrapRaw};

    use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
    use serde::ser::{Serialize, Serializer};

    struct ArrayVisitor<T: Type>(PhantomData<Il2CppArray<T>>);

    impl<'de, T: Type> ArrayVisitor<T>
    where
        T::Held<'de>: Deserialize<'de>,
    {
        fn visit_self<A>(mut seq: A, len: usize) -> Result<&'de mut Il2CppArray<T>, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let arr = unsafe { raw::array_new(T::class().raw(), len) }.unwrap();
            let data_ptr = ((arr as *mut _ as isize) + (raw::kIl2CppSizeOfArray as isize))
                as *mut T::Held<'de>;
            for i in 0..len {
                unsafe {
                    let ptr = data_ptr.add(i);
                    ptr::write_unaligned(ptr, seq.next_element()?.unwrap());
                }
            }
            Ok(unsafe { Il2CppArray::wrap_mut(arr) })
        }

        fn visit_vec<A>(mut seq: A) -> Result<Vec<T::Held<'de>>, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(elem) = seq.next_element()? {
                vec.push(elem);
            }
            Ok(vec)
        }
    }

    impl<'de, T: Type> Visitor<'de> for ArrayVisitor<T>
    where
        T::Held<'de>: Deserialize<'de>,
    {
        type Value = &'de mut Il2CppArray<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an array of C# compatible values")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            match seq.size_hint() {
                Some(len) => Self::visit_self(seq, len),
                None => Self::visit_vec(seq).map(Il2CppArray::new),
            }
        }
    }

    impl<'de, T: Type> Deserialize<'de> for &'de mut Il2CppArray<T>
    where
        T::Held<'de>: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_seq(ArrayVisitor(Default::default()))
        }
    }

    impl<T: Type> Serialize for Il2CppArray<T>
    where
        for<'a> T::Held<'a>: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            <[T::Held<'_>] as Serialize>::serialize(self.as_slice(), serializer)
        }
    }
}
