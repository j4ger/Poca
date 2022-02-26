use poca_server::include_app_dir;

fn main() {
    // note for myself:
    // to avoid using `let` in macro quotes,
    // design only one function that takes a vec of (string,bytes,option<(vec)>) tuples
    let app_routes = include_app_dir!("examples/public/");
    println!("Routes: {:?}", app_routes);
}
