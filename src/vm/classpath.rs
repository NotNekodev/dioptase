use std::path::PathBuf;

pub struct ClassPath {
    directories: Vec<PathBuf>,
}

impl ClassPath {
    pub fn empty() -> Self {
        Self {
            directories: Vec::new(),
        }
    }

    pub fn parse(cp_str: &str) -> Self {
        Self {
            directories: std::env::split_paths(cp_str).collect(),
        }
    }

    pub fn find_class_file(&self, binary_name: &str) -> Option<PathBuf> {
        for dir in &self.directories {
            let mut path = dir.clone();
            for component in binary_name.split('/') {
                path.push(component);
            }
            path.set_extension("class");
            if path.is_file() {
                return Some(path);
            }
        }
        None
    }
}
