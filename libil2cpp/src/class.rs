use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::{fmt, ptr, slice};

use crate::{
    raw, Arguments, CalleeReturn, CalleeThis, FieldInfo, Il2CppException, Il2CppType, MethodInfo,
    Parameters, Return, WrapRaw,
};

/// An il2cpp class
#[repr(transparent)]
pub struct Il2CppClass(raw::Il2CppClass);

impl Il2CppClass {
    /// Find a class by namespace and name
    pub fn find(
        namespace: impl Into<Cow<'static, str>>,
        name: impl Into<Cow<'static, str>>,
    ) -> Option<&'static Self> {
        // Reduce the amount of generated code by removing the generics
        fn inner(
            namespace: Cow<'static, str>,
            name: Cow<'static, str>,
        ) -> Option<&'static Il2CppClass> {
            #[cfg(feature = "cache")]
            let cache::ClassCacheKey { namespace, name } = {
                let key = cache::ClassCacheKey { namespace, name };
                if let Some(class) = cache::CLASS_CACHE.with(|c| c.borrow().get(&key).copied()) {
                    return Some(class);
                }
                key
            };

            let c_namespace = CString::new(namespace.as_ref()).unwrap();
            let c_name = CString::new(name.as_ref()).unwrap();

            let domain = unsafe { raw::domain_get() };

            let mut assemblies_count = 0;
            let assemblies = unsafe { raw::domain_get_assemblies(domain, &mut assemblies_count) };

            for assembly in assemblies.iter().take(assemblies_count) {
                // For some reason, an assembly might not have an image
                let image = match unsafe { raw::assembly_get_image(assembly) } {
                    Some(image) => image,
                    None => continue,
                };

                let class =
                    unsafe { raw::class_from_name(image, c_namespace.as_ptr(), c_name.as_ptr()) };
                if let Some(class) = class {
                    // Ensure class is initialized
                    // TODO: Call Class::Init somehow
                    let _ = unsafe { raw::class_get_method_from_name(class, "".as_ptr(), 0) };

                    let class = unsafe { Il2CppClass::wrap(class) };

                    #[cfg(feature = "cache")]
                    cache::CLASS_CACHE.with(move |c| {
                        c.borrow_mut()
                            .insert(cache::ClassCacheKey { namespace, name }, class)
                    });

                    return Some(class);
                }
            }

            None
        }

        inner(namespace.into(), name.into())
    }

    /// Find a method belonging to the class or its parents by name with type
    /// checking
    pub fn find_method<A, R, const N: usize>(&self, name: &str) -> Option<&MethodInfo>
    where
        A: Arguments<N>,
        R: Return,
    {
        for c in self.hierarchy() {
            let mut matching = c
                .methods()
                .iter()
                .filter(|mi| {
                    mi.name() == name && A::matches(mi.parameters()) && R::matches(mi.return_ty())
                })
                .copied();

            match match matching.next() {
                // If we have no matches, we continue to the parent
                None => continue,
                Some(mi) => (mi, matching.next()),
            } {
                // If we have one match, we return it
                (mi, None) => return Some(mi),
                // If we have 2+ matches, we return None to avoid conflicts
                _ => return None,
            }
        }

        None
    }

    /// Find a `static` method belonging to the class by name with type checking
    pub fn find_static_method<A, R, const N: usize>(&self, name: &str) -> Option<&MethodInfo>
    where
        A: Arguments<N>,
        R: Return,
    {
        let mut matching = self
            .methods()
            .iter()
            .filter(|mi| {
                mi.name() == name
                    && mi.is_static()
                    && A::matches(mi.parameters())
                    && R::matches(mi.return_ty())
            })
            .copied();

        match (matching.next(), matching.next()) {
            // If we have one match, we return it
            (Some(mi), None) | (None, Some(mi)) => Some(mi),
            // If we have 2+ or zero matches, we return None
            _ => None,
        }
    }

    /// Find a method belonging to the class or its parents by name with type
    /// checking from a callee perspective
    pub fn find_method_callee<T, P, R, const N: usize>(&self, name: &str) -> Option<&MethodInfo>
    where
        T: CalleeThis,
        P: Parameters<N>,
        R: CalleeReturn,
    {
        for c in self.hierarchy() {
            let mut matching = c
                .methods()
                .iter()
                .filter(|mi| {
                    mi.name() == name
                        && T::matches(mi)
                        && P::matches(mi.parameters())
                        && R::matches(mi.return_ty())
                })
                .copied();

            match match matching.next() {
                // If we have no matches, we continue to the parent
                None => continue,
                Some(mi) => (mi, matching.next()),
            } {
                // If we have one match, we return it
                (mi, None) => return Some(mi),
                // If we have 2+ matches, we return None to avoid conflicts
                _ => return None,
            }
        }

        None
    }

    /// Find a method belonging to the class or its parents by name and
    /// parameter count, without type checking
    pub fn find_method_unchecked(
        &self,
        name: &str,
        parameters_count: usize,
    ) -> Option<&MethodInfo> {
        for c in self.hierarchy() {
            let mut matching = c
                .methods()
                .iter()
                .filter(|mi| mi.name() == name && mi.parameters().len() == parameters_count)
                .copied();

            match match matching.next() {
                // If we have no matches, we continue to the parent
                None => continue,
                Some(mi) => (mi, matching.next()),
            } {
                // If we have one match, we return it
                (mi, None) => return Some(mi),
                // If we have 2+ matches, we return None to avoid conflicts
                _ => return None,
            }
        }

        None
    }

    /// Find a field belonging to the class or its parents by name
    pub fn find_field(&self, name: &str) -> Option<&FieldInfo> {
        for c in self.hierarchy() {
            let mut matching = c.fields().iter().filter(|fi| fi.name() == name);

            match matching.next() {
                // If we have no matches, we continue to the parent
                None => continue,
                Some(fi) => return Some(fi),
            }
        }

        None
    }

    /// Invokes the `static` method with the given name using the given
    /// arguments, with type checking
    pub fn invoke<A, R, const N: usize>(
        &self,
        name: &str,
        args: A,
    ) -> Result<R, &mut Il2CppException>
    where
        A: Arguments<N>,
        R: Return,
    {
        let method = self.find_static_method::<A, R, N>(name).unwrap();
        unsafe { method.invoke_unchecked((), args) }
    }

    /// Invokes the `static void` method with the given name using the given
    /// arguments, with type checking
    pub fn invoke_void<A, const N: usize>(
        &self,
        name: &str,
        args: A,
    ) -> Result<(), &mut Il2CppException>
    where
        A: Arguments<N>,
    {
        let method = self.find_static_method::<A, (), N>(name).unwrap();
        unsafe { method.invoke_unchecked((), args) }
    }

    /// Name of the class
    pub fn name(&self) -> Cow<'_, str> {
        let name = self.raw().name;
        assert!(!name.is_null());
        unsafe { CStr::from_ptr(name) }.to_string_lossy()
    }

    /// Namespace containing the class
    pub fn namespace(&self) -> Cow<'_, str> {
        let namespace = self.raw().namespaze;
        assert!(!namespace.is_null());
        unsafe { CStr::from_ptr(namespace) }.to_string_lossy()
    }

    /// Methods of the class
    pub fn methods(&self) -> &[&MethodInfo] {
        let raw = self.raw();
        let methods = raw.methods;
        if !methods.is_null() {
            unsafe { slice::from_raw_parts(methods as _, raw.method_count as _) }
        } else {
            &[]
        }
    }

    /// Fields of the class
    pub fn fields(&self) -> &[FieldInfo] {
        let raw = self.raw();
        let fields = raw.fields;
        if !fields.is_null() {
            unsafe { slice::from_raw_parts(fields as _, raw.field_count as _) }
        } else {
            &[]
        }
    }

    /// Parent of the class, if it inherits from any
    pub fn parent(&self) -> Option<&Il2CppClass> {
        unsafe { Il2CppClass::wrap_ptr(self.raw().parent) }
    }

    /// Iterator over the class hierarchy, starting with the class itself
    pub fn hierarchy(&self) -> Hierarchy<'_> {
        Hierarchy {
            current: Some(self),
        }
    }

    /// Interfaces this class implements
    pub fn implemented_interfaces(&self) -> &[&Il2CppClass] {
        let raw = self.raw();
        let interfaces = raw.implementedInterfaces;
        if !interfaces.is_null() {
            unsafe { slice::from_raw_parts(interfaces as _, raw.interfaces_count as _) }
        } else {
            &[]
        }
    }

    /// Nested types of the class
    pub fn nested_types(&self) -> &[&Il2CppClass] {
        let raw = self.raw();
        unsafe { slice::from_raw_parts(raw.nestedTypes as _, raw.nested_type_count as _) }
    }

    /// Whether the class is assignable from `other`
    pub fn is_assignable_from(&self, other: &Il2CppClass) -> bool {
        unsafe { raw::class_is_assignable_from(self.raw(), other.raw()) }
    }

    /// [`Il2CppType`] of `this` for the class
    pub fn this_arg_ty(&self) -> &Il2CppType {
        unsafe { Il2CppType::wrap(&self.raw().this_arg) }
    }

    /// [`Il2CppType`] of byval arguments for the class
    pub fn byval_arg_ty(&self) -> &Il2CppType {
        unsafe { Il2CppType::wrap(&self.raw().byval_arg) }
    }
}

/// Iterator over a class hierarchy
#[derive(Debug)]
pub struct Hierarchy<'a> {
    current: Option<&'a Il2CppClass>,
}

unsafe impl WrapRaw for Il2CppClass {
    type Raw = raw::Il2CppClass;
}

impl<'a> Iterator for Hierarchy<'a> {
    type Item = &'a Il2CppClass;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(c) => {
                self.current = c.parent();
                Some(c)
            }
            None => None,
        }
    }
}

impl fmt::Debug for Il2CppClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let namespace = self.namespace();
        let name = self.name();
        f.debug_struct("Il2CppClass")
            .field("namespace", &namespace)
            .field("name", &name)
            .finish()
    }
}

impl fmt::Display for Il2CppClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let namespace = &*self.namespace();
        let name = &*self.name();
        match namespace {
            "" => f.write_str(name),
            _ => write!(f, "{}.{}", namespace, name),
        }
    }
}

impl PartialEq for Il2CppClass {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl<'a> From<&'a Il2CppType> for &'a Il2CppClass {
    fn from(ty: &'a Il2CppType) -> Self {
        ty.class()
    }
}

#[cfg(feature = "cache")]
mod cache {
    use std::borrow::Cow;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(PartialEq, Eq, Hash)]
    pub(super) struct ClassCacheKey {
        pub(super) namespace: Cow<'static, str>,
        pub(super) name: Cow<'static, str>,
    }

    thread_local! {
        pub(super) static CLASS_CACHE: RefCell<HashMap<ClassCacheKey, &'static super::Il2CppClass>> = Default::default();
    }
}
