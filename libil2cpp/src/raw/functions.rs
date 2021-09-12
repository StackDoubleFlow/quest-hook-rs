#![allow(missing_docs)]

use libloading::{Library, Symbol};
use quest_hook_proc_macros::il2cpp_functions;
use std::ffi::c_void;
use std::lazy::{SyncLazy, SyncOnceCell};
use std::os::raw::c_char;

use super::{
    FieldInfo, Il2CppArray, Il2CppAssembly, Il2CppClass, Il2CppDomain, Il2CppException,
    Il2CppImage, Il2CppMethodPointer, Il2CppObject, Il2CppReflectionMethod, Il2CppReflectionType,
    Il2CppString, Il2CppType, MethodInfo,
};

il2cpp_functions! {
    pub fn domain_get() -> &'static Il2CppDomain;
    pub fn domain_get_assemblies(domain: &Il2CppDomain, size: &mut usize) -> &'static [&'static Il2CppAssembly];
    pub fn assembly_get_image(assembly: &Il2CppAssembly) -> Option<&'static Il2CppImage>;
    pub fn class_from_name(image: &Il2CppImage, namespace: *const c_char, name: *const c_char) -> Option<&'static Il2CppClass>;
    pub fn class_from_il2cpp_type(ty: &Il2CppType) -> &'static Il2CppClass;
    pub fn class_from_system_type(ty: &Il2CppReflectionType) -> &'static Il2CppClass;
    pub fn class_is_assignable_from(class: &Il2CppClass, other_class: &Il2CppClass) -> bool;
    pub fn class_get_method_from_name(class: &Il2CppClass, name: *const c_char, args_count: u32) -> Option<&'static MethodInfo>;
    pub fn class_get_type(class: &Il2CppClass) -> &'static Il2CppType;
    pub fn field_set_value(obj: &mut Il2CppObject, field: &FieldInfo, value: *const c_void);
    pub fn field_get_value(obj: &mut Il2CppObject, field: &FieldInfo, value: *mut c_void);
    pub fn method_get_object(method: &MethodInfo, refclass: Option<&Il2CppClass>) -> &'static mut Il2CppReflectionMethod;
    pub fn method_get_from_reflection(method: &Il2CppReflectionMethod) -> &'static MethodInfo;
    pub fn method_is_generic(method: &MethodInfo) -> bool;
    pub fn array_new(element_class: &Il2CppClass, length: usize) -> Option<&'static mut Il2CppArray>;
    pub fn array_class_get(element_class: &Il2CppClass, rank: u32) -> &'static Il2CppClass;
    pub fn type_get_name(ty: &Il2CppType) -> *const c_char;
    pub fn type_get_object(ty: &Il2CppType) -> &'static mut Il2CppReflectionType;
    pub fn runtime_invoke(method: &MethodInfo, instance: *mut c_void, params: *mut *mut c_void, exception: &mut Option<&mut Il2CppException>) -> Option<&'static mut Il2CppObject>;
    pub fn string_new_len(s: *const c_char, len: u32) -> &'static mut Il2CppString;
    pub fn raise_exception(exc: &Il2CppException) -> !;
    pub fn resolve_icall(name: *const c_char) -> Il2CppMethodPointer;
    pub fn object_new(class: &Il2CppClass) -> &'static mut Il2CppObject;
}
