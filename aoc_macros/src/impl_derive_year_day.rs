use quote::quote;
use syn::parse_macro_input;

use crate::extensions::*;

pub fn derive_year_day(
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item: syn::Item = parse_macro_input!(item as syn::Item);

    let (ident, generics) = match &item {
        syn::Item::Enum(item) => (&item.ident, &item.generics),
        syn::Item::Struct(item) => (&item.ident, &item.generics),
        _ => panic!("#[derive(YearDay)] only supports structs/enums"),
    };

    let generic_params: Vec<_> = generics
        .params
        .iter()
        .map(|param| param.without_default())
        .collect();
    let generic_args: Vec<_> = generics
        .params
        .iter()
        .map(|param| param.into_argument())
        .collect();
    let predicates: Vec<_> = generics
        .where_clause
        .iter()
        .flat_map(|clause| clause.predicates.iter())
        .collect();

    let stream = quote! {
        impl< #(#generic_params),* >
            ::aoc::YearDay for #ident < #(#generic_args),* >
        where
            #(#predicates,)*
        {
            fn year() -> u32 {
                let dirname = std::path::Path::new(std::file!())
                    .parent()
                    .and_then(|parent| parent.file_name())
                    .and_then(|dirname| dirname.to_str())
                    .expect("Couldn't find year#### directory");
                assert!(dirname.starts_with("year"));
                u32::from_str_radix(&dirname[4..], 10)
                    .expect("Couldn't parse directory as year####")
            }
            fn day() -> u8 {
                let stem = std::path::Path::new(std::file!())
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .expect("Couldn't parse filename as UTF-8");
                assert!(stem.starts_with("day"));
                u8::from_str_radix(&stem[3..], 10)
                    .expect("Couldn't parse filename as day##.rs")
            }
        }

    };
    let impl_item: syn::Item =
        syn::parse2(stream.clone()).unwrap_or_else(|err| {
            panic!(
                "Error while parsing generated YearDay impl: {err}\n{stream}"
            )
        });

    quote! { #impl_item }.into()
}
