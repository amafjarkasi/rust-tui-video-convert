use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct FileBrowser {
    current_dir: PathBuf,
    files: Vec<PathBuf>,
    selected_idx: usize,
    filter: Vec<String>,
}

impl FileBrowser {
    pub fn new(starting_dir: PathBuf) -> Self {
        let mut browser = Self {
            current_dir: starting_dir,
            files: Vec::new(),
            selected_idx: 0,
            filter: vec!["mp4", "mkv", "avi", "mov", "webm"].into_iter().map(String::from).collect(),
        };
        browser.refresh_files();
        browser
    }

    pub fn refresh_files(&mut self) {
        self.files.clear();
        
        // Add parent directory option if not at root
        if let Some(parent) = self.current_dir.parent() {
            self.files.push(parent.to_path_buf());
        }
        
        // Add directories and filtered files
        for entry in WalkDir::new(&self.current_dir)
            .max_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path().to_path_buf();
            
            // Skip the current directory (it's already represented)
            if path == self.current_dir {
                continue;
            }
            
            // Always include directories
            if path.is_dir() {
                self.files.push(path);
            } 
            // Only include files that match our filter
            else if let Some(extension) = path.extension() {
                if let Some(ext_str) = extension.to_str() {
                    if self.filter.contains(&ext_str.to_lowercase()) {
                        self.files.push(path);
                    }
                }
            }
        }
        
        // Reset selection
        self.selected_idx = 0;
    }
    
    pub fn next(&mut self) {
        if !self.files.is_empty() {
            self.selected_idx = (self.selected_idx + 1) % self.files.len();
        }
    }
    
    pub fn previous(&mut self) {
        if !self.files.is_empty() {
            if self.selected_idx > 0 {
                self.selected_idx -= 1;
            } else {
                self.selected_idx = self.files.len() - 1;
            }
        }
    }
    
    pub fn enter_directory(&mut self) -> bool {
        if self.files.is_empty() {
            return false;
        }
        
        let selected = &self.files[self.selected_idx];
        if selected.is_dir() {
            self.current_dir = selected.clone();
            self.refresh_files();
            true
        } else {
            false
        }
    }
    
    pub fn get_selected_file(&self) -> Option<&PathBuf> {
        if self.files.is_empty() {
            None
        } else {
            Some(&self.files[self.selected_idx])
        }
    }
    
    pub fn get_files(&self) -> &Vec<PathBuf> {
        &self.files
    }
    
    pub fn get_selected_idx(&self) -> usize {
        self.selected_idx
    }
    
    pub fn get_current_dir(&self) -> &PathBuf {
        &self.current_dir
    }
    
    pub fn is_selected_parent_dir(&self) -> bool {
        if self.files.is_empty() {
            return false;
        }
        
        if let Some(parent) = self.current_dir.parent() {
            return self.files[self.selected_idx] == parent.to_path_buf();
        }
        
        false
    }
    
    pub fn is_selected_file(&self) -> bool {
        if self.files.is_empty() {
            return false;
        }
        
        self.files[self.selected_idx].is_file()
    }
    
    pub fn format_path_for_display(&self, path: &Path) -> String {
        if let Some(parent) = self.current_dir.parent() {
            if path == parent {
                return "..".to_string();
            }
        }
        
        if path.is_dir() {
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    return format!("📁 {}", name_str);
                }
            }
            return "📁 <unknown>".to_string();
        } else {
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    return format!("🎬 {}", name_str);
                }
            }
            return "🎬 <unknown>".to_string();
        }
    }
}