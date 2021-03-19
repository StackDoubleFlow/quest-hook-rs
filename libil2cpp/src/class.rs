use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::{fmt, ptr, slice};

use crate::{raw, Il2CppType, MethodInfo, WrapRaw};

/// An il2cpp class
#[repr(transparent)]
pub struct Il2CppClass(raw::Il2CppClass);

impl Il2CppClass {
    /// Find a class by namespace and name
    pub fn find(namespace: &str, name: &str) -> Option<&'static Self> {
        let namespace = CString::new(namespace).unwrap();
        let name = CString::new(name).unwrap();

        let domain = raw::domain_get();

        let mut assemblies_count = 0;
        let assemblies = raw::domain_get_assemblies(domain, &mut assemblies_count);

        for assembly in assemblies.iter().take(assemblies_count) {
            // For some reason, an assembly might not have an image
            let image = match raw::assembly_get_image(assembly) {
                Some(image) => image,
                None => continue,
            };

            let class = raw::class_from_name(image, namespace.as_ptr(), name.as_ptr());
            if let Some(class) = class {
                return Some(unsafe { Self::wrap(class) });
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
                // If we have two matches, we return None to avoid conflicts
                _ => return None,
            }
        }

        None
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
        raw::class_is_assignable_from(self.raw(), other.raw())
    }
}

/// Iterator over the parents of a class
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
