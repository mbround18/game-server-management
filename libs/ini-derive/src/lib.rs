extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr, parse_macro_input};

struct IniHeaderArgs {
    name: LitStr,
}

impl syn::parse::Parse for IniHeaderArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Expect the key "name"
        let ident: syn::Ident = input.parse()?;
        if ident != "name" {
            return Err(syn::Error::new(ident.span(), "expected `name`"));
        }
        // Parse the '=' token
        input.parse::<syn::Token![=]>()?;
        // Parse a literal string value
        let lit: LitStr = input.parse()?;
        Ok(IniHeaderArgs { name: lit })
    }
}

#[proc_macro_derive(IniSerialize, attributes(INIHeader))]
pub fn ini_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Look for #[INIHeader(name = "...")]
    let mut header_value = None;
    for attr in input.attrs.iter() {
        if attr.path().is_ident("INIHeader") {
            if let Ok(args) = attr.parse_args::<IniHeaderArgs>() {
                header_value = Some(args.name.value());
            }
        }
    }

    let header_value = header_value.expect(
        "INIHeader attribute is required with a name, e.g. #[INIHeader(name = \"/Script/Pal.PalGameWorldSettings\")]",
    );

    let expanded = quote! {
        impl IniHeader for #name {
            fn ini_header() -> &'static str {
                #header_value
            }
        }

        // Also implement IniHeader for a reference to this type.
        impl IniHeader for & #name {
            fn ini_header() -> &'static str {
                <#name as IniHeader>::ini_header()
            }
        }
    };

    TokenStream::from(expanded)
}
