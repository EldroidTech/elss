use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;

pub mod site_builder {
    use super::*;

    pub struct SiteBuilder {
        dest_dir: PathBuf,
        src_dir: PathBuf,
        components_dir: String,
        layout_dir: String,
        cache: HashMap<PathBuf, String>,
        component_regex: Regex,
        layout_regex: Regex,
        layout_content_regex: Regex,
    }

    impl SiteBuilder {
        pub fn new(base_dir: PathBuf) -> Self {
            SiteBuilder {
                dest_dir: base_dir.join("build"),
                src_dir: base_dir.join("src"),
                components_dir: "el-components".to_string(),
                layout_dir: "el-layouts".to_string(),
                cache: HashMap::new(),
                component_regex: Regex::new(r#"<el-component\s+name="([^"]*)"\s*>(.*?)</el-component>"#).unwrap(),
                layout_regex: Regex::new(r#"<el-layout\s+name="([^"]*)"\s*>(.*?)</el-layout>"#).unwrap(),
                layout_content_regex: Regex::new(r#"<el-content\s*/>"#).unwrap(),
            }
        }

        fn flatten_file(&mut self, file: &Path) {
            let mut processing = HashSet::new();
            let result = self.replace_components(&file, &mut processing);
            let result = self.replace_layout(&result);
            let dest_path = self.dest_dir.join(file);
            if let Some(parent) = dest_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("Failed to create directory: {}", e);
                    return;
                }
            }
            if let Err(e) = fs::write(dest_path, result) {
                eprintln!("Failed to write file: {}", e);
            }
        }

        pub fn build(&mut self) {
            if let Err(e) = fs::remove_dir_all(&self.dest_dir) {
                eprintln!("Failed to remove directory: {}", e);
            }
            let src_dir = self.src_dir.clone();
            self.process_files(&src_dir);
        }

        fn copy_to_output(&self, path: &Path) {
            let dest_path = self.dest_dir.join(&path);
            let src_path = self.src_dir.join(&path);
            if let Some(parent) = dest_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("Failed to create directory: {}", e);
                    return;
                }
            }
            if let Err(e) = fs::copy(src_path, dest_path) {
                eprintln!("Failed to copy file [{}]: {}", path.to_str().unwrap_or_default(), e);
            }
        }

        fn directory_to_ignore(&self, path: &Path) -> bool {
            path.starts_with(&self.components_dir) || path.starts_with(&self.layout_dir)
        }

        fn process_file(&mut self, path: &Path) {
            if path.extension().and_then(|s| s.to_str()) == Some("html") {
                self.flatten_file(&path);
            } else {
                self.copy_to_output(&path);
            }
        }

        fn process_files(&mut self, path: &Path) {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(Result::ok) {
                    let entry_path = entry.path();
                    let entry_path_relative = entry_path.strip_prefix(&self.src_dir).unwrap();
                    if entry_path.is_dir() {
                        if self.directory_to_ignore(&entry_path_relative) {
                            continue;
                        }
                        self.process_files(&entry_path);
                    } else {
                        self.process_file(&entry_path_relative);
                    }
                }
            }
        }

        fn replace_components(&mut self, path: &Path, processing: &mut HashSet<String>) -> String {
            let dest_path = self.dest_dir.join(path);
    
            if let Some(file_contents) = self.cache.get(&dest_path) {
                return file_contents.clone();
            }
    
            let src_path = self.src_dir.join(path);
            let text = match fs::read_to_string(&src_path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Failed to read file [{}]: {}", src_path.display(), e);
                    return String::new();
                }
            };
            let mut result = text.clone();
    
            let captures: Vec<_> = self.component_regex.captures_iter(&text).collect();
            for captures in captures {
                if let Some(src) = captures.get(1) {
                    let component_path = format!("{}/{}", self.components_dir, src.as_str().trim_end_matches(".html").to_string() + ".html");
                    
                    if processing.contains(&component_path) {
                        eprintln!("Circular dependency detected for component [{}]", component_path);
                        continue;
                    }
    
                    processing.insert(component_path.clone());
                    let file_contents = self.replace_components(Path::new(&component_path), processing);
                    processing.remove(&component_path);
    
                    result = result.replace(&captures[0], &file_contents);
                }
            }
    
            self.cache.insert(dest_path, result.clone());
            result
        }
        
        fn replace_layout(&mut self, content: &str) -> String {
            let mut result = content.to_string();
            if let Some(captures) = self.layout_regex.captures(content) {
                if let Some(src) = captures.get(1) {
                    let file_path = format!("{}/{}", self.layout_dir, src.as_str().trim_end_matches(".html").to_string() + ".html");
                    let mut processing = HashSet::new();
                    let file_contents = self.replace_components(Path::new(&file_path), &mut processing);
                    let layout_content = self.layout_content_regex.replace_all(&file_contents, &captures[2]);
                    result = layout_content.to_string();
                }
            }
            result
        }
    }
}