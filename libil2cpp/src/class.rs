use super::{raw, MethodInfo, WrapRaw};
use std::ffi::{CStr, CString};
use std::slice;

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

    /// Find a method belonging to the class by name and parameter count
    pub fn find_method(&self, name: &str, parameters_count: usize) -> Option<&MethodInfo> {
        self.methods().iter().copied().find(|mi| {
            mi.name().to_string_lossy() == name && mi.parameters_count() == parameters_count
        })
    }

    /// Name of the class
    pub fn name(&self) -> &CStr {
        let name = self.raw().name;
        assert!(!name.is_null());
        unsafe { CStr::from_ptr(name) }
    }

    /// Namespace containing the class
    pub fn namespace(&self) -> &CStr {
        let namespace = self.raw().namespaze;
        assert!(!namespace.is_null());
        unsafe { CStr::from_ptr(namespace) }
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

    /// Class the class inherits from, if it inherits from any
    pub fn parent(&self) -> Option<&Il2CppClass> {
        unsafe { Il2CppClass::wrap_ptr(self.raw().parent) }
    }

    /// Nested types of the class
    pub fn nested_types(&self) -> &[&Il2CppClass] {
        let raw = self.raw();
        unsafe { slice::from_raw_parts(raw.nestedTypes as _, raw.nested_type_count as _) }
    }
}

unsafe impl WrapRaw for Il2CppClass {
    type Raw = raw::Il2CppClass;
}
