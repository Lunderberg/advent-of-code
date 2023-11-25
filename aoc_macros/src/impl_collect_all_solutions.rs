use itertools::Itertools;
use quote::{format_ident, quote};

pub fn collect_all_solutions() -> proc_macro::TokenStream {
    let cargo_dir: std::path::PathBuf =
        std::env::var("CARGO_MANIFEST_DIR").unwrap().into();

    ////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////
    ////
    //// This would be the cleaner solution, with absolute paths to each day's
    //// puzzle.  However, rust-analyzer breaks whenever the `#[path = "..."]`
    //// attribute is used.  If this issue is resolved, this implementation can
    //// be used instead of the current one.
    ////
    //// See https://github.com/rust-lang/rust-analyzer/issues/15965
    ////
    ////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////

    // let (mod_idents, mod_path_strings): (Vec<_>, Vec<_>) = cargo_dir
    //     .join("src")
    //     .read_dir()
    //     .unwrap()
    //     .map(|res| res.unwrap().path())
    //     .filter(|path| {
    //         path.is_dir()
    //             && path
    //                 .file_name()
    //                 .expect("Last segment should not be ..")
    //                 .to_str()
    //                 .expect("Cannot convert path to UTF-8")
    //                 .starts_with("year")
    //     })
    //     .flat_map(|year_path| {
    //         year_path
    //             .read_dir()
    //             .unwrap()
    //             .map(|res| res.unwrap().path())
    //             .filter(|path| {
    //                 path.is_file()
    //                     && path
    //                         .file_name()
    //                         .expect("Last segment should not be ..")
    //                         .to_str()
    //                         .expect("Cannot convert path to UTF-8")
    //                         .starts_with("day")
    //             })
    //     })
    //     .sorted()
    //     .map(|path| {
    //         let year_str = path
    //             .parent()
    //             .unwrap()
    //             .file_name()
    //             .unwrap()
    //             .to_str()
    //             .unwrap();
    //         let day_str = path.file_stem().unwrap().to_str().unwrap();
    //         let mod_ident = format_ident!("{year_str}_{day_str}");

    //         let mod_path_string = format!("{}", path.display());

    //         (mod_ident, mod_path_string)
    //     })
    //     .unzip();

    // let stream = quote! {
    //    mod solution_detail {
    //        #(
    //            #[path = #mod_path_strings]
    //            mod #mod_idents;
    //        )*

    //        pub fn solutions() -> impl Iterator<Item=Box<dyn ::aoc::framework::PuzzleRunner>> {
    //            [
    //                #(
    //                    ::aoc::framework::PuzzleRunnerImpl::< #mod_idents :: ThisDay>::new_box(),
    //                )*
    //            ].into_iter()
    //        }
    //    }
    //    pub use solution_detail::solutions;
    // };

    ////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////
    ////
    //// The slightly messier solution used instead.  The module listing needs
    //// to exactly reproduce the directory hierarchy.
    ////
    ////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////

    let solutions_by_year = cargo_dir
        .join("src")
        .read_dir()
        .unwrap()
        .map(|res| res.unwrap().path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .expect("Last segment should not be ..")
                    .to_str()
                    .expect("Cannot convert path to UTF-8")
                    .starts_with("year")
        })
        .flat_map(|year_path| {
            year_path
                .read_dir()
                .unwrap()
                .map(|res| res.unwrap().path())
                .filter(|path| {
                    path.is_file()
                        && path
                            .file_name()
                            .expect("Last segment should not be ..")
                            .to_str()
                            .expect("Cannot convert path to UTF-8")
                            .starts_with("day")
                })
        })
        .sorted()
        .map(|path| {
            let year_str = path
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            let day_str = path.file_stem().unwrap().to_str().unwrap();
            (format_ident!("{year_str}"), format_ident!("{day_str}"))
        })
        .into_group_map();

    let year_mods = solutions_by_year.iter().map(|(year, days)| {
        quote! {
            mod #year {
                #( pub mod #days; )*
            }
        }
    });

    let all_puzzles = solutions_by_year
        .iter()
        .flat_map(|(year, days)| days.iter().map(move |day| (year, day)))
        .map(|(year, day)| {
            quote! {
                ::aoc::framework::PuzzleRunnerImpl::<
                    #year :: #day :: ThisDay
                >::new_box()
            }
        });

    let stream = quote! {
        #(#year_mods)*

        pub fn solutions() -> impl Iterator<Item=Box<dyn ::aoc::framework::PuzzleRunner>> {
            [ #(#all_puzzles,)* ].into_iter()
        }
    };

    stream.into()
}
