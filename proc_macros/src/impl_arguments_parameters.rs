use std::num::NonZeroUsize;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, Expr, ExprLit, ExprRange, Index, Lit, RangeLimits};

pub fn expand(range: ExprRange) -> Result<TokenStream, Error> {
    let span = range.span();

    let start = range
        .from
        .ok_or_else(|| Error::new(span, "Tuple length range must have a lower bound"))?;
    let start = parse_range_bound(*start)?;

    let end = range
        .to
        .ok_or_else(|| Error::new(span, "Tuple length range must have an upper bound"))?;
    let end = parse_range_bound(*end)?;

    let range = match range.limits {
        RangeLimits::HalfOpen(_) if end <= start => {
            return Err(Error::new(span, "Tuple length range must be valid"))
        }
        RangeLimits::HalfOpen(_) => start..end,

        RangeLimits::Closed(_) if end < start => {
            return Err(Error::new(span, "Tuple length range must be valid"))
        }
        RangeLimits::Closed(_) => start..(end + 1),
    };

    let mut ts = TokenStream::new();
    for n in range {
        let generic_params_argument = (1..=n).map(|n| format_ident!("A{}", n));
        let matches_argument = generic_params_argument
            .clone()
            .enumerate()
            .map(|(n, gp)| quote!(<#gp>::matches(params.get_unchecked(#n).ty())));
        let invokables = (0..n).map(Index::from).map(|n| quote!(self.#n.invokable()));

        let generic_params_parameter = (1..=n).map(|n| format_ident!("P{}", n));
        let matches_parameter = generic_params_parameter
            .clone()
            .enumerate()
            .map(|(n, gp)| quote!(<#gp>::matches(params.get_unchecked(#n).ty())));

        let generic_params_argument_tuple = generic_params_argument.clone();
        let generic_params_argument_where = generic_params_argument.clone();
        let generic_params_argument_type = generic_params_argument.clone();

        let generic_params_parameter_tuple = generic_params_parameter.clone();
        let generic_params_parameter_where = generic_params_parameter.clone();

        let impl_ts = quote! {
            unsafe impl<#(#generic_params_argument),*> Arguments<#n> for (#(#generic_params_argument_tuple,)*)
            where
                #(#generic_params_argument_where: Argument),*
            {
                type Type = (#(#generic_params_argument_type::Type,)*);

                fn matches(method: &MethodInfo) -> bool {
                    let params = method.parameters();
                    params.len() == #n && unsafe { #(#matches_argument) && * }
                }

                fn invokable(&mut self) -> [*mut c_void; #n] {
                    [#(#invokables),*]
                }
            }

            unsafe impl<#(#generic_params_parameter),*> Parameters for (#(#generic_params_parameter_tuple,)*)
            where
                #(#generic_params_parameter_where: Parameter),*
            {
                const COUNT: usize = #n;

                fn matches(method: &MethodInfo) -> bool {
                    let params = method.parameters();
                    params.len() == #n && unsafe { #(#matches_parameter) && * }
                }
            }
        };
        ts.extend(TokenStream::from(impl_ts));
    }

    Ok(ts)
}

fn parse_range_bound(bound: Expr) -> Result<usize, Error> {
    let bound: NonZeroUsize = match bound {
        syn::Expr::Lit(ExprLit {
            lit: Lit::Int(n), ..
        }) => n.base10_parse()?,
        _ => {
            return Err(Error::new(
                bound.span(),
                "Tuple length bound must be an integer",
            ))
        }
    };
    Ok(bound.get())
}
