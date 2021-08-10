use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Error, FnArg, ForeignItemFn, LitByteStr, PatType, Signature};

pub struct Input(Vec<ForeignItemFn>);

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self, Error> {
        let mut fns = Vec::new();
        while !input.is_empty() {
            fns.push(input.parse::<ForeignItemFn>()?);
        }
        Ok(Self(fns))
    }
}

pub fn expand(input: &Input) -> Result<TokenStream, Error> {
    let mut ts = quote! {
        static LIBIL2CPP: SyncLazy<Library> =
            SyncLazy::new(|| unsafe { Library::new("libil2cpp.so") }.unwrap());
    };

    for ForeignItemFn {
        attrs,
        vis,
        sig:
            Signature {
                ident,
                inputs,
                output,
                ..
            },
        ..
    } in input.0.iter()
    {
        let name = LitByteStr::new(format!("il2cpp_{}", ident).as_bytes(), ident.span());

        let inputs = inputs
            .iter()
            .map(|i| match i {
                FnArg::Receiver(_) => {
                    Err(Error::new_spanned(i, "il2cpp functions cannot take `self`"))
                }
                FnArg::Typed(p) => Ok(p),
            })
            .collect::<Result<Vec<&PatType>, Error>>()?;
        let inputs = inputs.as_slice();

        let input_pats = inputs.iter().map(|i| &i.pat);
        let input_tys = inputs.iter().map(|i| &i.ty);

        let wrapper = quote! {
            #(#attrs) *
            #vis unsafe fn #ident(#(#inputs),*) #output {
                static FN: SyncOnceCell<Symbol<'static, unsafe extern "C" fn(#(#input_tys),*) #output>> =
                    SyncOnceCell::new();
                let fun = FN.get_or_init(|| unsafe { LIBIL2CPP.get(#name) }.unwrap());
                (**fun)(#(#input_pats),*)
            }
        };
        ts.extend_one(wrapper);
    }

    Ok(ts.into())
}
