mod impl_derive_year_day;

pub(crate) mod extensions;

#[proc_macro_derive(YearDay)]
pub fn derive_year_day(
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    impl_derive_year_day::derive_year_day(item)
}
