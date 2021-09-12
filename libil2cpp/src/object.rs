use std::fmt;
use std::ops::DerefMut;

use crate::{raw, Argument, Arguments, Il2CppClass, Il2CppException, Returned, Type, WrapRaw};

/// An il2cpp object
#[repr(transparent)]
pub struct Il2CppObject(raw::Il2CppObject);

impl Il2CppObject {
    /// [`Il2CppClass`] of the object
    pub fn class(&self) -> &'static Il2CppClass {
        unsafe { Il2CppClass::wrap_ptr(self.raw().__bindgen_anon_1.klass) }.unwrap()
    }

    /// Invokes the method with the given name on `self` using the given
    /// arguments, with type checking
    ///
    /// # Panics
    ///
    /// This method will panic if a matching method can't be found.
    pub fn invoke<A, R, const N: usize>(
        &mut self,
        name: &str,
        args: A,
    ) -> Result<R, &mut Il2CppException>
    where
        A: Arguments<N>,
        R: Returned,
    {
        let method = self.class().find_method::<A, R, N>(name).unwrap();
        unsafe { method.invoke_unchecked(self, args) }
    }

    /// Invokes the `void` method with the given name on `self` using the
    /// given arguments, with type checking
    ///
    /// # Panics
    ///
    /// This method will panic if a matching method can't be found.
    pub fn invoke_void<A, const N: usize>(
        &mut self,
        name: &str,
        args: A,
    ) -> Result<(), &mut Il2CppException>
    where
        A: Arguments<N>,
    {
        let method = self.class().find_method::<A, (), N>(name).unwrap();
        unsafe { method.invoke_unchecked(self, args) }
    }

    /// Loads a value from a field of `self` with the given name, with type
    /// checking
    ///
    /// # Panics
    ///
    /// This method will panic if the given field can't be found
    pub fn load<T>(&mut self, field: &str) -> T::Held<'_>
    where
        T: Type,
    {
        let field = self.class().find_field(field).unwrap();
        field.load::<T>(self)
    }

    /// Stores a given value into a field of `self` with the given name, with
    /// type checking
    ///
    /// # Panics
    ///
    /// This method will panic if the given field can't be found
    pub fn store<A>(&mut self, field: &str, value: A)
    where
        A: Argument,
    {
        let field = self.class().find_field(field).unwrap();
        field.store(self, value);
    }
}

unsafe impl WrapRaw for Il2CppObject {
    type Raw = raw::Il2CppObject;
}

impl fmt::Debug for Il2CppObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Il2CppObject")
            .field("class", self.class())
            .finish()
    }
}

/// Helper trait for reference types which can be dereferenced to an object
#[rustfmt::skip]
pub trait ObjectExt:
    for<'a> Type<Held<'a> = Option<&'a mut Self>> + DerefMut<Target = Il2CppObject> + Sized
{
    /// Creates a new object using the constructor taking the given arguments
    fn new<A, const N: usize>(args: A) -> &'static mut Self
    where
        A: Arguments<N>,
    {
        let object: &mut Self = Self::class().instanciate();
        object.invoke_void(".ctor", args).unwrap();
        object
    }
}
#[rustfmt::skip]
impl<T> ObjectExt for T
where
    for<'a> T: Type<Held<'a> = Option<&'a mut Self>>,
    T: DerefMut<Target = Il2CppObject>,
{
}
