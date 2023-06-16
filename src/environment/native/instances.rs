use std::sync::Arc;

use super::super::types::Instance;

#[derive(Clone)]
pub struct Instances {
    instances: Arc<Vec<Instance>>,
}

impl Default for Instances {
    fn default() -> Self {
        Self {
            instances: Arc::new(
                serde_json::from_slice(include_bytes!("../instances.json")).unwrap(),
            ),
        }
    }
}

impl Instances {
    /// Find in our list of known instances
    pub async fn search(&self, term: Option<String>) -> Vec<Instance> {
        let count = 50;
        let Some(term) = term else {
            return self.instances.iter().take(count).cloned().collect()
        };

        self.instances
            .iter()
            .filter(|o| o.name.contains(&term))
            .take(count)
            .cloned()
            .collect()
    }
}
