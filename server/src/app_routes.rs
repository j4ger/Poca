#[derive(Debug)]
pub struct AppRoutes<'a> {
    pub root: &'a str,
    pub routes: Vec<AppRoutes<'a>>,
    pub content: &'a [u8],
}

// A route triplet is a path-segment + content in &[u8] + subroutes (if any)
pub enum RouteNode<'a> {
    E(&'a str, &'a [u8]),                          //EndPoint
    S(&'a str, &'a [u8], Box<Vec<RouteNode<'a>>>), //SplitPoint
}

pub fn generate_app_routes(routes: RouteNode) -> AppRoutes {
    let (root, content, sub_routes) = match routes {
        RouteNode::E(root, content) => (root, content, Vec::new()),
        RouteNode::S(root, content, sub_routes) => (root, content, *sub_routes),
    };

    AppRoutes {
        root,
        content,
        routes: sub_routes.into_iter().map(generate_app_routes).collect(),
    }
}
