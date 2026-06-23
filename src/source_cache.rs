use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct SourceCache {
    sources: Arc<RwLock<HashMap<String, Arc<String>>>>,
}

impl SourceCache {
    pub fn new() -> Self {
        Self {
            sources: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_source(&self, filename: String, source: String) {
        let mut sources = self.sources.write().unwrap();
        sources.insert(filename, Arc::new(source));
    }

    pub fn get_source(&self, filename: &str) -> Option<Arc<String>> {
        let sources = self.sources.read().unwrap();
        sources.get(filename).cloned()
    }

    pub fn has_source(&self, filename: &str) -> bool {
        let sources = self.sources.read().unwrap();
        sources.contains_key(filename)
    }

    pub fn clear(&self) {
        let mut sources = self.sources.write().unwrap();
        sources.clear();
    }
}

impl Default for SourceCache {
    fn default() -> Self {
        Self::new()
    }
}
