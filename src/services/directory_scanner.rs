use std::path::{Path, PathBuf};
use std::fs;

const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "bmp", "tiff"];

pub fn get_images_in_dir(dir: &Path) -> Vec<PathBuf> {
    let mut images = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                        images.push(path);
                    }
                }
            }
        }
    }
    
    // Natural alphanumeric sorting is ideal, but standard lexical sorting works for Phase 1
    images.sort();
    images
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_directory_discovery() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        File::create(dir_path.join("image1.jpg")).unwrap();
        File::create(dir_path.join("text.txt")).unwrap();
        File::create(dir_path.join("image2.PNG")).unwrap();

        let images = get_images_in_dir(dir_path);
        assert_eq!(images.len(), 2);
        
        // Ensure unsupported files are skipped
        assert!(!images.contains(&dir_path.join("text.txt")));
    }
}