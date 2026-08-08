use proc_macro::TokenStream;
use quote::quote;
use syn::{
    ItemFn, LitStr, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct NativeArgs {
    class: LitStr,
    name: LitStr,
    descriptor: LitStr,
}

impl Parse for NativeArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut class = None;
        let mut name = None;
        let mut descriptor = None;

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;

            match ident.to_string().as_str() {
                "class" => class = Some(value),
                "name" => name = Some(value),
                "descriptor" => descriptor = Some(value),
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        "unknown native attribute argument",
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            class: class.ok_or_else(|| syn::Error::new(input.span(), "missing `class`"))?,
            name: name.ok_or_else(|| syn::Error::new(input.span(), "missing `name`"))?,
            descriptor: descriptor
                .ok_or_else(|| syn::Error::new(input.span(), "missing `descriptor`"))?,
        })
    }
}

#[proc_macro_attribute]
pub fn native(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as NativeArgs);
    let function = parse_macro_input!(item as ItemFn);

    let function_name = &function.sig.ident;

    let class = args.class;
    let name = args.name;
    let descriptor = args.descriptor;

    let output = quote! {
        #function

        inventory::submit! {
            crate::native::native_registry::NativeRegistration {
                class_name: #class,
                method_name: #name,
                descriptor: #descriptor,
                function: #function_name,
            }
        }
    };

    output.into()
}
