use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::{fmt, ptr, slice};

use crate::{
    raw, Arguments, FieldInfo, Il2CppException, Il2CppType, MethodInfo, Parameters, Return,
    Returned, ThisParameter, WrapRaw,
};

#[cfg(feature = "unity2019")]
type FieldInfoSlice<'a> = &'a [FieldInfo];
#[cfg(feature = "unity2018")]
type FieldInfoSlice<'a> = &'a [&'static FieldInfo];

/// An il2cpp class
#[repr(transparent)]
pub struct Il2CppClass(raw::Il2CppClass);

impl Il2CppClass {
    /// Find a class by namespace and name
    pub fn find(namespace: &str, name: &str) -> Option<&'static Self> {
        #[cfg(feature = "cache")]
        let key = {
            let key = cache::ClassCacheKey {
                namespace: namespace.into(),
                name: name.into(),
            };
            if let Some(class) = cache::CLASS_CACHE.with(|c| c.borrow().get(&key).copied()) {
                return Some(class);
            }
            key
        };

        let c_namespace = CString::new(namespace).unwrap();
        let c_name = CString::new(name).unwrap();

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
                cache::CLASS_CACHE.with(move |c| c.borrow_mut().insert(key.into(), class));

                return Some(class);
            }
        }

        None
    }

    /// Find a method belonging to the class or its parents by name with type
    /// checking
    pub fn find_method<A, R, const N: usize>(
        &self,
        name: &str,
    ) -> Result<&'static MethodInfo, FindMethodError>
    where
        A: Arguments<N>,
        R: Returned,
    {
        #[cfg(feature = "cache")]
        let key = {
            let class_key = cache::ClassCacheKey {
                namespace: self.namespace(),
                name: self.name(),
            };
            let key = cache::MethodCacheKey {
                class: class_key,
                name: name.into(),
                ty: std::any::TypeId::of::<fn(Self, A::Type) -> R::Type>(),
            };
            if let Some(method) = cache::METHOD_CACHE.with(|c| c.borrow().get(&key).copied()) {
                return Ok(method);
            }
            key
        };

        for c in self.hierarchy() {
            let mut matching = c
                .methods()
                .iter()
                .filter(|mi| mi.name() == name && A::matches(mi) && R::matches(mi.return_ty()))
                .copied();

            match match matching.next() {
                // If we have no matches, we continue to the parent
                None => continue,
                Some(mi) => (mi, matching.next()),
            } {
                (mi, None) => {
                    #[cfg(feature = "cache")]
                    cache::METHOD_CACHE.with(move |c| c.borrow_mut().insert(key.into(), mi));

                    return Ok(mi);
                }
                _ => return Err(FindMethodError::Many),
            }
        }

        Err(FindMethodError::None)
    }

    /// Find a `static` method belonging to the class by name with type checking
    pub fn find_static_method<A, R, const N: usize>(
        &self,
        name: &str,
    ) -> Result<&'static MethodInfo, FindMethodError>
    where
        A: Arguments<N>,
        R: Returned,
    {
        #[cfg(feature = "cache")]
        let key = {
            let class_key = cache::ClassCacheKey {
                namespace: self.namespace(),
                name: self.name(),
            };
            let key = cache::MethodCacheKey {
                class: class_key,
                name: name.into(),
                ty: std::any::TypeId::of::<fn((), A::Type) -> R::Type>(),
            };
            if let Some(method) = cache::METHOD_CACHE.with(|c| c.borrow().get(&key).copied()) {
                return Ok(method);
            }
            key
        };

        for c in self.hierarchy() {
            let mut matching = c
                .methods()
                .iter()
                .filter(|mi| {
                    mi.name() == name
                        && mi.is_static()
                        && A::matches(mi)
                        && R::matches(mi.return_ty())
                })
                .copied();

            match match matching.next() {
                // If we have no matches, we continue to the parent
                None => continue,
                Some(mi) => (mi, matching.next()),
            } {
                (mi, None) => {
                    #[cfg(feature = "cache")]
                    cache::METHOD_CACHE.with(move |c| c.borrow_mut().insert(key.into(), mi));

                    return Ok(mi);
                }
                _ => return Err(FindMethodError::Many),
            }
        }

        Err(FindMethodError::None)
    }

    /// Find a method belonging to the class or its parents by name with type
    /// checking from a callee perspective
    pub fn find_method_callee<T, P, R>(
        &self,
        name: &str,
    ) -> Result<&'static MethodInfo, FindMethodError>
    where
        T: ThisParameter,
        P: Parameters,
        R: Return,
    {
        let mut matching = self
            .methods()
            .iter()
            .filter(|mi| {
                mi.name() == name && T::matches(mi) && P::matches(mi) && R::matches(mi.return_ty())
            })
            .copied();

        match (matching.next(), matching.next()) {
            (Some(mi), None) | (None, Some(mi)) => Ok(mi),
            (Some(_), Some(_)) => Err(FindMethodError::Many),
            (None, None) => Err(FindMethodError::None),
        }
    }

    /// Find a method belonging to the class or its parents by name and
    /// parameter count, without type checking
    pub fn find_method_unchecked(
        &self,
        name: &str,
        parameters_count: usize,
    ) -> Result<&'static MethodInfo, FindMethodError> {
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
                (mi, None) => return Ok(mi),
                _ => return Err(FindMethodError::Many),
            }
        }

        Err(FindMethodError::None)
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
        R: Returned,
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
    pub fn methods(&self) -> &[&'static MethodInfo] {
        let raw = self.raw();
        let methods = raw.methods;
        if !methods.is_null() {
            unsafe { slice::from_raw_parts(methods as _, raw.method_count as _) }
        } else {
            &[]
        }
    }

    /// Fields of the class
    pub fn fields(&self) -> FieldInfoSlice<'_> {
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

    /// [`Il2CppType`] of the class
    pub fn ty(&self) -> &Il2CppType {
        unsafe { Il2CppType::wrap(raw::class_get_type(self.raw())) }
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

/// Possible errors when looking up a method
#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FindMethodError {
    /// No matching method were found
    #[error("no matching methods found")]
    None,

    /// Multiple matching methods were found
    #[error("multiple matching methods found")]
    Many,
}

#[cfg(feature = "cache")]
mod cache {
    use std::any::TypeId;
    use std::borrow::{Borrow, Cow};
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(PartialEq, Eq, Hash)]
    pub(super) struct ClassCacheKey<'a> {
        pub(super) namespace: Cow<'a, str>,
        pub(super) name: Cow<'a, str>,
    }

    #[derive(PartialEq, Eq, Hash)]
    pub(super) struct StaticClassCacheKey(ClassCacheKey<'static>);

    impl<'a> From<ClassCacheKey<'a>> for StaticClassCacheKey {
        fn from(ClassCacheKey { namespace, name }: ClassCacheKey<'a>) -> Self {
            let namespace = namespace.into_owned().into();
            let name = name.into_owned().into();
            Self(ClassCacheKey { namespace, name })
        }
    }

    impl<'a> Borrow<ClassCacheKey<'a>> for StaticClassCacheKey {
        fn borrow(&self) -> &ClassCacheKey<'a> {
            &self.0
        }
    }

    #[derive(PartialEq, Eq, Hash)]
    pub(super) struct MethodCacheKey<'a> {
        pub(super) class: ClassCacheKey<'a>,
        pub(super) name: Cow<'a, str>,
        pub(super) ty: TypeId,
    }

    #[derive(PartialEq, Eq, Hash)]
    pub(super) struct StaticMethodCacheKey(MethodCacheKey<'static>);

    impl<'a> From<MethodCacheKey<'a>> for StaticMethodCacheKey {
        fn from(MethodCacheKey { class, name, ty }: MethodCacheKey<'a>) -> Self {
            let class = StaticClassCacheKey::from(class).0;
            let name = name.into_owned().into();
            Self(MethodCacheKey { class, name, ty })
        }
    }

    impl<'a> Borrow<MethodCacheKey<'a>> for StaticMethodCacheKey {
        fn borrow(&self) -> &MethodCacheKey<'a> {
            &self.0
        }
    }

    thread_local! {
        pub(super) static CLASS_CACHE: RefCell<HashMap<StaticClassCacheKey, &'static super::Il2CppClass>> = Default::default();
        pub(super) static METHOD_CACHE: RefCell<HashMap<StaticMethodCacheKey, &'static super::MethodInfo>> = Default::default();
    }
}
