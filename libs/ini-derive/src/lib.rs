//! # ini-derive
//!
//! Procedural macro crate that provides `#[derive(IniSerialize)]` for use with
//! `gsm-serde`.
//!
//! ## Usage
//!
//! Annotate a struct with `#[derive(IniSerialize)]` and supply the section header
//! via `#[INIHeader(name = "...")]`:
//!
//! ```rust,ignore
//! use ini_derive::IniSerialize;
//! use gsm_serde::serde_ini::IniHeader;
//!
//! #[derive(serde::Serialize, IniSerialize)]
//! #[INIHeader(name = "/Script/Pal.PalGameWorldSettings")]
//! struct GameSettings {
//!     difficulty: String,
//! }
//!
//! assert_eq!(GameSettings::ini_header(), "/Script/Pal.PalGameWorldSettings");
//! ```
//!
//! The derive macro implements [`gsm_serde::serde_ini::IniHeader`] for both the struct
//! and a shared reference to it.
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

/// Derives [`gsm_serde::serde_ini::IniHeader`] for the annotated struct.
///
/// Requires the `#[INIHeader(name = "...")]` attribute on the struct to specify the
/// INI section header string.
///
/// # Panics
///
/// Panics at compile time if the `#[INIHeader(name = "...")]` attribute is absent.
#[proc_macro_derive(IniSerialize, attributes(INIHeader))]
pub fn ini_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Look for #[INIHeader(name = "...")]
    let mut header_value = None;
    for attr in input.attrs.iter() {
        if attr.path().is_ident("INIHeader")
            && let Ok(args) = attr.parse_args::<IniHeaderArgs>()
        {
            header_value = Some(args.name.value());
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
