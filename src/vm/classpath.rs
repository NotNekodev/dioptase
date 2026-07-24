use std::{fs::File, io::Read, path::PathBuf};

use zip::ZipArchive;

enum ClassPathEntry {
    Directory(PathBuf),
    Jar(ZipArchive<File>),
}

pub struct ClassPath {
    bootstrap: Vec<ClassPathEntry>,
    extension: Vec<ClassPathEntry>,
    application: Vec<ClassPathEntry>,
}

impl ClassPath {
    pub fn empty() -> Self {
        Self {
            bootstrap: Vec::new(),
            extension: Vec::new(),
            application: Vec::new(),
        }
    }

    pub fn parse(cp: &str) -> Self {
        Self {
            bootstrap: Vec::new(),
            extension: Vec::new(),
            application: Self::parse_entries(cp),
        }
    }

    fn parse_entries(cp: &str) -> Vec<ClassPathEntry> {
        std::env::split_paths(cp)
            .filter_map(|path| Self::parse_entry(path))
            .collect()
    }

    pub fn add_bootstrap(&mut self, path: PathBuf) {
        if let Some(entry) = Self::parse_entry(path) {
            self.bootstrap.push(entry);
        }
    }

    pub fn add_extension(&mut self, path: PathBuf) {
        if let Some(entry) = Self::parse_entry(path) {
            self.extension.push(entry);
        }
    }

    fn parse_entry(path: PathBuf) -> Option<ClassPathEntry> {
        if path.is_dir() {
            Some(ClassPathEntry::Directory(path))
        } else if path.extension().is_some_and(|e| e == "jar" || e == "zip") {
            let file = File::open(&path).ok()?;
            let archive = ZipArchive::new(file).ok()?;

            Some(ClassPathEntry::Jar(archive))
        } else {
            None
        }
    }

    fn read_entry(entry: &mut ClassPathEntry, name: &str) -> Option<Vec<u8>> {
        match entry {
            ClassPathEntry::Directory(dir) => {
                let path = dir.join(name);

                if !path.is_file() {
                    return None;
                }

                std::fs::read(path).ok()
            }

            ClassPathEntry::Jar(zip) => {
                let mut file = zip.by_name(name).ok()?;

                let mut data = Vec::new();

                file.read_to_end(&mut data).ok()?;

                Some(data)
            }
        }
    }

    pub fn find_class(&mut self, binary_name: &str) -> Option<Vec<u8>> {
        let name = format!("{}.class", binary_name);

        for entry in self
            .bootstrap
            .iter_mut()
            .chain(self.extension.iter_mut())
            .chain(self.application.iter_mut())
        {
            if let Some(data) = Self::read_entry(entry, &name) {
                return Some(data);
            }
        }

        None
    }
}
