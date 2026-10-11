use std::path::{Path, PathBuf};

pub struct GalleryState {
    pub images: Vec<PathBuf>,
    pub current_index: Option<usize>,
}

impl GalleryState {
    pub fn new(images: Vec<PathBuf>, current_image: Option<&Path>) -> Self {
        let mut state = GalleryState {
            images,
            current_index: None,
        };
        if let Some(current) = current_image {
            state.set_current_by_path(current);
        }
        state
    }

    pub fn set_current_by_path(&mut self, path: &Path) {
        self.current_index = self.images.iter().position(|p| p == path);
    }

    pub fn current_image(&self) -> Option<&PathBuf> {
        self.current_index.and_then(|idx| self.images.get(idx))
    }

    pub fn next(&mut self) -> Option<&PathBuf> {
        if let Some(idx) = self.current_index {
            if idx + 1 < self.images.len() {
                self.current_index = Some(idx + 1);
                return self.current_image();
            }
        } else if !self.images.is_empty() {
            self.current_index = Some(0);
            return self.current_image();
        }
        None
    }

    pub fn prev(&mut self) -> Option<&PathBuf> {
        if let Some(idx) = self.current_index {
            if idx > 0 {
                self.current_index = Some(idx - 1);
                return self.current_image();
            }
        } else if !self.images.is_empty() {
            self.current_index = Some(self.images.len() - 1);
            return self.current_image();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_gallery() {
        let mut state = GalleryState::new(vec![], None);
        assert_eq!(state.next(), None);
        assert_eq!(state.prev(), None);
        assert_eq!(state.current_image(), None);
    }

    #[test]
    fn test_navigation_boundaries() {
        let paths = vec![
            PathBuf::from("1.jpg"),
            PathBuf::from("2.png"),
            PathBuf::from("3.webp"),
        ];
        let mut state = GalleryState::new(paths, Some(Path::new("1.jpg")));
        
        assert_eq!(state.current_index, Some(0));
        assert_eq!(state.prev(), None); // Boundary: disabled prev at start
        
        assert_eq!(state.next(), Some(&PathBuf::from("2.png")));
        assert_eq!(state.next(), Some(&PathBuf::from("3.webp")));
        assert_eq!(state.next(), None); // Boundary: disabled next at end
    }
}