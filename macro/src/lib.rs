use std::path::Path;

use poca_server::AppRoutes;
use proc_macro::TokenStream;
use quote::quote_spanned;

#[proc_macro]
pub fn include_app_dir(item: TokenStream) -> TokenStream {
    let item = item.to_string();
    let input: Vec<&str> = item.split(",").collect();
    let default_file_name;
    let dir_path = input[0].trim_matches('"');
    match input.len() {
        0 => {
            panic!("No path provided")
        }
        1 => default_file_name = vec!["index.html", "index.htm"],
        _ => {
            default_file_name = input[1..].to_vec();
        }
    }
    // should be something like "/foo/bar"
    let directory = Path::new(&dir_path);
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = Path::new(&root).join(directory);
    if !full_path.exists() {
        panic!("{} does not exist", full_path.display());
    }
    if !full_path.is_dir() {
        panic!("{} is not a directory", full_path.display());
    }
    let full_path = full_path.to_string_lossy();
    let span = proc_macro2::Span::call_site();
    quote_spanned! {span=>
        let _directory_map = poca_server::traverse_directory("", #full_path);
        let _content_map:std::collections::HashMap<String,Vec<u8>> = HashMap::new();
        for _path in _directory_map.1 {
            let _content = include_bytes!(_path);
            _content_map.insert(_path, _content.to_vec());
        }
        let _app_routes = poca_server::generate_app_routes("",_directory_map.0,_content_map,vec![#(#default_file_name),*]);
        _app_routes
    }
    .into()
}
