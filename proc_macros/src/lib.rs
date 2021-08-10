#![feature(box_patterns, extend_one)]
#![doc(html_root_url = "https://stackdoubleflow.github.io/quest-hook-rs/quest_hook_proc_macros")]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::string_add_assign,
    clippy::string_add,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::use_self,
    clippy::useless_transmute,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    missing_docs,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]

//! Procedural macros for the quest_hook crate

mod hook;
mod il2cpp_functions;
mod impl_arguments_parameters;

use proc_macro::TokenStream;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, ExprRange, ItemFn, LitStr, Token};

/// Creates an inline hook at a C# method.
///
/// # Panics
///
/// * `original` will panic if the hook has not yet been installed.
#[proc_macro_attribute]
pub fn hook(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args =
        parse_macro_input!(attr with Punctuated<LitStr, Token![,]>::parse_separated_nonempty);
    let input = parse_macro_input!(item as ItemFn);

    match hook::expand(&args, input) {
        Ok(ts) => ts,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
#[doc(hidden)]
pub fn il2cpp_functions(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as il2cpp_functions::Input);
    match il2cpp_functions::expand(&input) {
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

#[proc_macro_attribute]
#[doc(hidden)]
pub fn identity(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}
