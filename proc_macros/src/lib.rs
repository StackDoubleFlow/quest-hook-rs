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
mod impl_generics;
mod impl_type;

use std::num::NonZeroUsize;
use std::ops::Range;

use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Error, Expr, ExprLit, ExprRange, ItemFn, Lit, LitStr, RangeLimits, Result,
    Token,
};

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

/// Implements the `Type` trait for a Rust type that is equivalent to a C#
/// reference type
///
/// Note that in order to use this macros, the `generic_associated_types`
/// feature must be enabled.
///
/// # Safety
///
/// The type must hold the guarantees required by the `Type` trait.
///
/// # Examples
///
/// The basic syntax follows the pattern `in <libil2cpp path> for <Rust type> =>
/// <C# type>`.
///
/// ```ignore
/// #![feature(generic_associated_types)]
///
/// use libil2cpp::Il2CppObject;
///
/// #[repr(C)]
/// struct GameObject {
///     object: Il2CppObject,
/// }
///
/// unsafe_impl_reference_type!(in libil2cpp for GameObject => UnityEngine.GameObject);
/// ```
///
/// It's also possible to use this macro with generic types. In this scenario,
/// the `once_cell` feature must also be enabled.
///
/// ```ignore
/// #![feature(generic_associated_types, once_cell)]
///
/// use libil2cpp::{Il2CppArray, Il2CppObject, Type};
///
/// #[repr(C)]
/// struct List<T: Type> {
///     object: Il2CppObject,
///     items: *mut Il2CppArray<T>,
///     size: i32,
/// }
///
/// unsafe_impl_reference_type!(in libil2cpp for List<T> => System.Collections.Generic.List<T>);
/// ```
///
/// A class getter can be provided manually.
///
/// ```ignore
/// #![feature(generic_associated_types)]
///
/// use libil2cpp::Il2CppObject;
///
/// #[repr(C)]
/// struct MyClass {
///     object: Il2CppObject,
/// }
///
/// unsafe_impl_reference_type!(in libil2cpp for MyClass => MyNamespace.MyClass {
///     my_class_getter()
/// });
///
/// # fn my_class_getter() -> &'static libil2cpp::Il2CppClass { unimplemented!() }
/// ```
///
/// Finally, the namespace and class name can be provided as string literals. In
/// this case, the macro will no change them in any way regardless of generic
/// parameters.
///
/// ```ignore
/// #![feature(generic_associated_types, once_cell)]
///
/// use libil2cpp::{Il2CppArray, Il2CppObject, Type};
///
/// #[repr(C)]
/// struct List<T: Type> {
///     object: Il2CppObject,
///     items: *mut Il2CppArray<T>,
///     size: i32,
/// }
///
/// unsafe_impl_reference_type!(in libil2cpp for List<T> => "System.Collections.Generic"."List`1"<T>);
/// ```
#[proc_macro]
pub fn unsafe_impl_reference_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as impl_type::Input);
    impl_type::expand(&input, impl_type::Semantics::Reference)
}

/// Implements the `Type` trait for a Rust type that is equivalent to a C# value
/// type
///
/// Note that in order to use this macros, the `generic_associated_types` and
/// `once_cell` features must be enabled.
///
/// This macro works the same way as [`unsafe_impl_reference_type!`], except
/// that it is meant for value types and implements some extra traits not
/// covered by blanket impls.
///
/// # Safety
///
/// The type must hold the guarantees required by the `Type` trait.
///
/// # Examples
///
/// The basic syntax follows the pattern `in <libil2cpp path> for <Rust type> =>
/// <C# type>`.
///
/// ```ignore
/// #![feature(generic_associated_types)]
///
/// #[repr(C)]
/// struct Vector3 {
///     x: f32,
///     y: f32,
///     z: f32,
/// }
///
/// unsafe_impl_value_type!(in libil2cpp for Vector3 => UnityEngine.Vector3);
/// ```
#[proc_macro]
pub fn unsafe_impl_value_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as impl_type::Input);
    impl_type::expand(&input, impl_type::Semantics::Value)
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
    let range = parse_macro_input!(input as RangeInput);
    match impl_arguments_parameters::expand(range.0) {
        Ok(ts) => ts,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
#[doc(hidden)]
pub fn impl_generics(input: TokenStream) -> TokenStream {
    let range = parse_macro_input!(input as RangeInput);
    match impl_generics::expand(range.0) {
        Ok(ts) => ts,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
#[doc(hidden)]
pub fn identity(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}

struct RangeInput(Range<usize>);

impl Parse for RangeInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let range: ExprRange = input.parse()?;
        let span = range.span();

        let start = range
            .from
            .ok_or_else(|| Error::new(span, "tuple length range must have a lower bound"))?;
        let start = parse_range_bound(*start)?;

        let end = range
            .to
            .ok_or_else(|| Error::new(span, "tuple length range must have an upper bound"))?;
        let end = parse_range_bound(*end)?;

        let range = match range.limits {
            RangeLimits::HalfOpen(_) if end <= start => {
                return Err(Error::new(span, "tuple length range must be valid"))
            }
            RangeLimits::HalfOpen(_) => start..end,

            RangeLimits::Closed(_) if end < start => {
                return Err(Error::new(span, "tuple length range must be valid"))
            }
            RangeLimits::Closed(_) => start..(end + 1),
        };
        Ok(Self(range))
    }
}

fn parse_range_bound(bound: Expr) -> Result<usize> {
    let bound: NonZeroUsize = match bound {
        Expr::Lit(ExprLit {
            lit: Lit::Int(n), ..
        }) => n.base10_parse()?,
        _ => {
            return Err(Error::new(
                bound.span(),
                "tuple length bound must be an integer",
            ))
        }
    };
    Ok(bound.get())
}
