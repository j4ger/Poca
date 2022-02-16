use std::{collections::HashMap, fs::read_dir, path::PathBuf};

#[derive(Debug)]
pub struct AppRoutes {
    pub root: String,
    pub routes: Vec<AppRoutes>,
    pub content: Vec<u8>,
}

#[derive(Debug)]
pub struct Routes {
    root: String,
    path: PathBuf,
    routes: Vec<Routes>,
}

pub fn traverse_directory(root: &str, directory: PathBuf) -> (Routes, Vec<String>) {
    let root = format!(
        "{}/{}",
        root,
        directory
            .file_name()
            .expect(format!("Failed to get filename for {}", directory.to_str().unwrap()).as_str())
            .to_string_lossy()
    );
    let mut result = Routes {
        root: root.clone(),
        path: directory.clone(),
        routes: Vec::new(),
    };
    let mut list = Vec::new();
    for entry in read_dir(directory).unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                let inner = traverse_directory(&root, path);
                result.routes.push(inner.0);
                list.extend(inner.1);
            } else {
                let file_name = path.file_name().unwrap().to_string_lossy();
                let full_path = format!("{}/{}", root, file_name);
                result.routes.push(Routes {
                    root: full_path.clone(),
                    routes: Vec::new(),
                    path: path.clone(),
                });
                list.push(path.to_string_lossy().to_string());
            }
        } else {
            panic!("{}", entry.unwrap_err());
        }
    }
    (result, list)
}

pub fn generate_app_routes(
    root: String,
    routes: Routes,
    content_map: &HashMap<String, Vec<u8>>,
    default_route: &Vec<&str>,
) -> AppRoutes {
    let mut result = AppRoutes {
        root: root.clone(),
        routes: Vec::new(),
        content: content_map
            .get(&routes.path.to_string_lossy().to_string())
            .unwrap_or(&Vec::new())
            .clone(),
    };
    if routes.routes.len() == 0 {
        let content = content_map
            .get(&routes.path.to_string_lossy().to_string())
            .unwrap_or(&Vec::new())
            .clone();
        if default_route.contains(
            &routes
                .path
                .file_name()
                .expect("Failed to read filename")
                .to_str()
                .expect("Failed to read filename"),
        ) {
            result.content = content.clone();
            let sub_root = format!(
                "{}/{}",
                &root,
                &routes
                    .path
                    .file_name()
                    .expect("Failed to read filename")
                    .to_string_lossy()
            );
            result.routes.push(AppRoutes {
                root: sub_root,
                routes: Vec::new(),
                content,
            });
        }
    } else {
        for route in routes.routes {
            let sub_root = format!(
                "{}/{}",
                &root,
                &route
                    .path
                    .file_name()
                    .expect("Failed to read filename")
                    .to_string_lossy()
            );
            result.routes.push(generate_app_routes(
                sub_root,
                route,
                content_map,
                default_route,
            ));
        }
    }
    result
}
