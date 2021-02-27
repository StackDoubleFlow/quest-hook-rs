use crate::libil2cpp::functions as il2cpp_functions;
use crate::libil2cpp::types::Il2CppClass;
use std::ffi::CString;

pub fn find_class(namespace: &str, class_name: &str) -> Option<&'static Il2CppClass> {
    let namespace = CString::new(namespace).unwrap();
    let class_name = CString::new(class_name).unwrap();

    let domain = il2cpp_functions::domain_get();

    let mut assemblies_count = 0;
    let assemblies = il2cpp_functions::domain_get_assemblies(domain, &mut assemblies_count);

    for assembly in assemblies.iter().take(assemblies_count) {
        // For some reason, an assembly might not have an image
        let image = il2cpp_functions::assembly_get_image(assembly);
        if image.is_none() {
            continue;
        }

        let class = il2cpp_functions::class_from_name(
            image.unwrap(),
            namespace.as_ptr(),
            class_name.as_ptr(),
        );
        if let Some(class) = class {
            return Some(class);
        }
    }

    None
}
