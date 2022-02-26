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
    pub fn get_route(&self, path: &[&str], initial: bool) -> Option<&'a [u8]> {
        if path.len() == 0 {
            return None;
        }
        if path[0] == "" {
            return Some(self.content);
        }
        if path[0] == self.root || initial {
            if path.len() == 1 {
                return Some(self.content);
            } else {
                let next_path = if initial { path } else { &path[1..] };
                return self
                    .routes
                    .iter()
                    .find_map(|route| route.get_route(next_path, false));
            }
        } else {
            return None;
        }
    }
}
