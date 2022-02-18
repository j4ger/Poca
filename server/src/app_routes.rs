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

impl<'a> AppRoutes<'a> {
    pub fn get_route(&self, path: &[&str]) -> Option<&'a [u8]> {
        if path.len() == 0 {
            return None;
        }
        if path[0] == "" || path[0] == self.root {
            return Some(self.content);
        }
        return self.routes.iter().find_map(|route| route.get_route(path));
    }
}
