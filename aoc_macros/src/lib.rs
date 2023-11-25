mod impl_collect_all_solutions;
mod impl_derive_year_day;

pub(crate) mod extensions;

#[proc_macro_derive(YearDay)]
pub fn derive_year_day(
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    impl_derive_year_day::derive_year_day(item)
}

#[proc_macro]
pub fn collect_all_solutions(
    _item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    impl_collect_all_solutions::collect_all_solutions()
}
