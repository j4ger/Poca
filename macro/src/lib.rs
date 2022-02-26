use std::{
    env,
    fs::read_dir,
    path::{Path, PathBuf},
};

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};

#[proc_macro]
pub fn include_app_dir(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = item.to_string();
    let input = input.split(",").collect::<Vec<&str>>();

    let dir_input = input.get(0).expect("No path provided").trim_matches('\"');
    let path = Path::new(&dir_input);
    let project_root = env::var("CARGO_MANIFEST_DIR")
        .expect("Failed to resolve CARGO_MANIFEST_DIR environment variable");
    let full_path = Path::new(&project_root).join(&path);

    let default_file_name = match input.len() {
        1 => {
            vec!["index.html", "index.htm"]
        }
        _ => input[1..].to_vec(),
    };

    let routes;

    if full_path.exists() {
        if full_path.is_dir() {
            routes = process_directory(full_path, &default_file_name);
        } else {
            routes = process_file(full_path);
        }
    } else {
        panic!("Path {:?} does not exist", full_path);
    }

    let span = Span::call_site();

    quote_spanned! {span=>
        poca::_g_a_r(#routes)
    }
    .into()
}

fn process_file(path: PathBuf) -> TokenStream {
    let file_name = path
        .file_name()
        .expect(format!("Failed to get filename for {:?}", &path).as_str())
        .to_string_lossy()
        .to_string();
    let path = path.to_string_lossy().to_string();

    quote! {
        poca::_N::E(#file_name,include_bytes!(#path))
    }
}

fn process_directory(path: PathBuf, default_file_name: &Vec<&str>) -> TokenStream {
    let file_name = path
        .file_name()
        .expect(format!("Failed to get filename for {:?}", &path).as_str())
        .to_string_lossy()
        .to_string();
    let path = path.to_string_lossy().to_string();

    let mut default_content = quote! {
        &[]
    };

    let mut result = Vec::new();

    for sub_entry in
        read_dir(&path).expect(format!("Failed to read directory:{:?}", &path).as_str())
    {
        if let Ok(sub_entry) = sub_entry {
            let sub_file_name = sub_entry.file_name().to_string_lossy().to_string();
            let sub_file_path = sub_entry.path();
            let sub_file_path_string = sub_file_path.to_string_lossy().to_string();

            if default_file_name.contains(&sub_file_name.as_str()) {
                default_content = quote! {
                    include_bytes!(#sub_file_path_string)
                };
            }
            if sub_file_path.is_dir() {
                result.push(process_directory(sub_file_path, default_file_name));
            } else {
                result.push(process_file(sub_file_path));
            }
        }
    }

    quote! {
        poca::_N::S(#file_name,#default_content,Box::new(vec![#(#result),*]))
    }
}
