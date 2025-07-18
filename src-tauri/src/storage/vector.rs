use color_eyre::eyre::Result;
use hnsw_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, path::Path, sync::Arc};
use tokio::sync::RwLock;
use tracing::instrument;
use uuid::{self, Uuid};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VectorEntry {
    pub id: Uuid,
    pub text: String,
    pub embedding: Vec<f32>,
    pub metadata: serde_json::Value,
}

impl Debug for VectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorStore")
            .field(
                "entries_count",
                &self.entries.try_read().map(|e| e.len()).unwrap_or(0),
            )
            .finish()
    }
}

pub struct VectorStore {
    index: Arc<RwLock<Hnsw<'static, f32, DistCosine>>>,
    entries: Arc<RwLock<Vec<VectorEntry>>>,
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            index: Arc::new(RwLock::new(Hnsw::new(
                16,      // M: number of bi-directional links
                200,     // ef_construction: size of dynamic candidate list
                128,     // dimension: embedding dimension
                1000000, // max_elements: maximum number of elements
                DistCosine,
            ))),
            entries: Arc::new(RwLock::new(Vec::new())),
        }
    }

    #[instrument(err, skip(self))]
    pub async fn store(&self, text: &str, embedding: &[f32]) -> Result<()> {
        let entry = VectorEntry {
            id: Uuid::new_v4(),
            text: text.to_string(),
            embedding: embedding.to_vec(),
            metadata: serde_json::json!({}),
        };
        // Add to HNSW index
        let index = self.index.write().await;
        let entry_idx = self.entries.read().await.len();
        index.insert((&entry.embedding, entry_idx));
        // Store entry
        self.entries.write().await.push(entry);
        Ok(())
    }

    #[instrument(err, skip(self))]
    pub async fn search(&self, query: &[f32], k: usize) -> Result<Vec<VectorEntry>> {
        let index = self.index.read().await;
        let entries = self.entries.read().await;
        let results = index.search(query, k, 100); // ef_search = 100
        let matches = results
            .into_iter()
            .filter_map(|result| entries.get(result.d_id).cloned())
            .collect();
        Ok(matches)
    }

    #[instrument(err, skip(self))]
    pub async fn delete(&self, id: &Uuid) -> Result<()> {
        let mut entries = self.entries.write().await;
        if let Some(pos) = entries.iter().position(|e| e.id == *id) {
            entries.remove(pos);
        }
        Ok(())
    }

    #[instrument(err, skip(self))]
    pub async fn save_to_disk(&self, path: impl AsRef<Path> + Debug) -> Result<()> {
        let path = path.as_ref();
        let entries = self.entries.read().await;
        let file = std::fs::File::create(path)?;
        serde_json::to_writer(file, &*entries)?;
        Ok(())
    }

    #[instrument(err, skip(self))]
    pub async fn load_from_disk(&self, path: &str) -> Result<()> {
        let file = std::fs::File::open(path)?;
        let entries: Vec<VectorEntry> = serde_json::from_reader(file)?;

        let index = Hnsw::new(16, 200, 128, 1000000, DistCosine);

        for (i, entry) in entries.iter().enumerate() {
            index.insert((&entry.embedding, i));
        }

        *self.index.write().await = index;
        *self.entries.write().await = entries;
        Ok(())
    }
}
