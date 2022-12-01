use std::fs::File;
use std::io::Write;
use std::path::Path;

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
                //(!input.is_empty()).then(|| input.parse())
                (!input.is_empty()).then(|| {
                    println!("Before");
                    let item: syn::Item = input.parse().unwrap();
                    println!("Item: {item:?}");
                    item
                })
            })
            //.collect::<Result<Vec<_>, _>>()?;
            .collect::<Vec<_>>();
            Ok(Self { items })

            // let mut result = Vec::new();
            // while !input.is_empty() {
            //     result.push(input.parse()?)
            // }
            // Ok(Self(result))
        }
    }

    let items: Items = syn::parse2(tokens).expect("Failed to parse/prettify");
    //let items = syn::parse2(tokens).expect("Failed to parse/prettify");
    let file = syn::File {
        attrs: vec![],
        items: items.items,
        shebang: None,
    };

    prettyplease::unparse(&file)
}

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let mut dst =
        File::create(Path::new(&out_dir).join("generated_year2022.rs"))
            .unwrap();

    let cargo_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mod_path = "src/year2022";

    // TODO: Search the directory given in "mod_path" to generate
    // "module_names".
    let module_names = vec!["day01"];

    let modules: Vec<_> = module_names
        .iter()
        .map(|name| format_ident!("{name}"))
        .collect();

    // The import would be relative to the generated file, rather than
    // being relative to the include location.  Therefore, giving an
    // absolute path to the imported modules.
    let file_names: Vec<_> = module_names
        .iter()
        .map(|name| format!("{cargo_dir}/{mod_path}/{name}.rs"))
        .collect();

    let stream = quote! {
        #(
            #[path = #file_names]
            mod #modules;
        )*

        use crate::framework::{PuzzleRunner, PuzzleRunnerImpl};

        pub fn solutions() -> impl Iterator<Item = Box<dyn PuzzleRunner>> {
            vec![
                #(
                    PuzzleRunnerImpl::<#modules::ThisPuzzle>::new_box(),
                )*
            ]
                .into_iter()
        }
    };

    //dst.write_all("const VALUE: usize = 0;".as_bytes()).unwrap();
    //dst.write_all(format!("{stream}").as_bytes()).unwrap();
    dst.write_all(pretty_print(stream).as_bytes()).unwrap();
}
