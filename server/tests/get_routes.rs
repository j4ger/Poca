#[cfg(test)]

mod tests {
    use poca::include_app_dir;

    #[test]
    fn get_route() {
        let routes = include_app_dir!("tests/routes_test/");
        let route = routes.get_route(&["layer1", "layer2", "layer3-1"], true);
        println!("{:?}", route);
    }
}
