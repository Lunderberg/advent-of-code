use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};

fn pretty_print(tokens: TokenStream) -> String {
    struct Items {
        items: Vec<syn::Item>,
    }
    impl Parse for Items {
        fn parse(input: ParseStream) -> Result<Self, syn::Error> {
            let items = std::iter::from_fn(|| {
                (!input.is_empty()).then(|| input.parse().unwrap())
            })
            .collect::<Vec<_>>();
            Ok(Self { items })
        }
    }

    let items: Items = syn::parse2(tokens).expect("Failed to parse/prettify");
    let file = syn::File {
        attrs: vec![],
        items: items.items,
        shebang: None,
    };

    prettyplease::unparse(&file)
}

fn generate(mod_path: &Path) -> String {
    let dir = mod_path.parent().unwrap();

    let module_paths: Vec<_> = dir
        .read_dir()
        .unwrap()
        .map(|io_res| io_res.unwrap().path())
        .filter(|path| {
            path.is_file()
                && path.file_name().unwrap() != "mod.rs"
                && path.file_name().unwrap() != "template.rs"
        })
        .collect();

    let modules: Vec<_> = module_paths
        .iter()
        .map(|path| {
            format_ident!("{}", path.file_stem().unwrap().to_str().unwrap())
        })
        .collect();

    let module_str_paths: Vec<_> = module_paths
        .iter()
        .map(|path| path.to_str().unwrap())
        .collect();

    let stream = quote! {
        #(
            #[path = #module_str_paths]
            mod #modules;
        )*

        use crate::framework::{PuzzleRunner, PuzzleRunnerImpl};

        pub fn solutions() -> impl Iterator<Item = Box<dyn PuzzleRunner>> {
            vec![
                #(
                    PuzzleRunnerImpl::<#modules::ThisDay>::new_box(),
                )*
            ]
                .into_iter()
        }
    };

    pretty_print(stream)
}

fn main() {
    let cargo_dir: PathBuf =
        std::env::var("CARGO_MANIFEST_DIR").unwrap().into();
    let mod_path = cargo_dir.join("src/year2022/mod.rs");

    let out_dir: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    let mut dst = File::create(out_dir.join("generated_year2022.rs")).unwrap();

    dst.write_all(generate(&mod_path).as_bytes()).unwrap();
}
