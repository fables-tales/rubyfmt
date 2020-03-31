//! Generates roughly the same code as `#[derive(Deserialize)] #[serde(untagged)]`,
//! but special cases the fact that we're deserializing from a `VALUE`, which is a
//! pointer that can be copied for free -- skipping serde's buffering

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(RipperDeserialize, attributes(tag))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let item: syn::ItemEnum = syn::parse(input).expect(
        "`#[derive(RipperDeserialize)]` only works on enums. Use \
         `#[derive(Serialize)]` for structs",
    );
    let enum_name = item.ident;

    let variant_exprs = item.variants.into_iter().map(|syn::Variant { ident, fields, .. }| {
        assert_eq!(fields.len(), 1, "#[derive(RipperDeserialize)] requires \
            all variants to have exactly one field");

        quote! {
            if let Ok(x) =  serde::Deserialize::deserialize(deserializer).map(#enum_name::#ident) {
                return Ok(x);
            }
        }
    });

    let tokens = quote! {
        impl<'de> serde::Deserialize<'de> for #enum_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use crate::ruby::VALUE;
                use serde::Deserialize;

                let value = VALUE::deserialize(deserializer)?;
                let deserializer = value.into_deserializer();

                #(#variant_exprs)*

                Err(serde::de::Error::custom(concat!(
                    "No variant matched untagged enum ",
                    stringify!(#enum_name),
                    ". (Error from ripper_deserialize)",
                )))
            }
        }
    };

    tokens.into()
}
