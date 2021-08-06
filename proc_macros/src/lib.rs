#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]
#![cfg_attr(feature = "strict", deny(warnings))]
#![feature(box_patterns, extend_one)]
#![doc(html_root_url = "https://stackdoubleflow.github.io/quest-hook-rs/quest_hook_proc_macros")]

//! Procedural macros for the quest_hook crate

mod hook;
mod impl_arguments_parameters;

use proc_macro::TokenStream;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, ExprRange, ItemFn, LitStr, Token};

/// Creates an inline hook at a C# method.
///
/// # Panics
///
/// * `install` will panic if the class or method was not found.
/// * `original` will panic if the hook has not yet been installed.
///
/// # Examples
///
/// ```no_run
/// use quest_hook::hook;
/// use quest_hook::libil2cpp::{Il2CppObject, Il2CppString};
/// use quest_hook::tracing::debug;
///
/// #[no_mangle]
/// pub extern "C" fn setup() {
///     quest_hook::setup("hello world");
/// }
///
/// #[hook("UnityEngine.SceneManagement", "SceneManager", "SetActiveScene")]
/// fn set_active_scene(scene: &mut Il2CppObject) -> bool {
///     let name: &Il2CppString = scene.invoke("get_name", ()).unwrap();
///     debug!("Hello, {}!", name);
///
///     set_active_scene.original(scene)
/// }
///
/// #[no_mangle]
/// pub extern "C" fn load() {
///     set_active_scene.install().unwrap();
/// }
/// ```
#[proc_macro_attribute]
pub fn hook(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args =
        parse_macro_input!(attr with Punctuated<LitStr, Token![,]>::parse_separated_nonempty);
    let input = parse_macro_input!(item as ItemFn);

    match hook::expand(args, input) {
        Ok(ts) => ts,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
#[doc(hidden)]
pub fn impl_arguments_parameters(input: TokenStream) -> TokenStream {
    let range = parse_macro_input!(input as ExprRange);
    match impl_arguments_parameters::expand(range) {
        Ok(ts) => ts,
        Err(err) => err.to_compile_error().into(),
    }
}
