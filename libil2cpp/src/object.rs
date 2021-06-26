use std::fmt;

use crate::{raw, Argument, Arguments, Il2CppClass, Il2CppException, Return, WrapRaw};

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
    ) -> Result<R, &Il2CppException>
    where
        A: Arguments<N>,
        R: Return,
    {
        let method = self.class().find_method::<A, R, N>(name).unwrap();
        unsafe { method.invoke_unchecked(self, args) }
    }

    /// Loads a value from a field of `self` with the given name, with type
    /// checking
    ///
    /// # Panics
    ///
    /// This method will panic if the given field can't be found
    pub fn load<R>(&mut self, field: &str) -> R
    where
        R: Return,
    {
        let field = self.class().find_field(field).unwrap();
        field.load(self)
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
        field.store(self, value)
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
