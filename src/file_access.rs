//! File access tracking for frecency-based path completion.
//!
//! Tracks file access patterns (frequency + recency) to prioritize
//! commonly/recently used files in completions.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const MAX_ENTRIES: usize = 500;
const MAX_COUNT: f64 = 100.0;
const MIN_SCORE_THRESHOLD: f64 = 0.5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessEntry {
    pub path: String,
    pub count: f64,
    pub last_accessed: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AccessData {
    #[serde(default)]
    accesses: Vec<AccessEntry>,
}

pub struct FileAccessTracker {
    entries: HashMap<String, AccessEntry>,
    config_path: std::path::PathBuf,
    dirty: bool,
}

impl FileAccessTracker {
    /// Load tracker from config file.
    pub fn load(config_path: &Path) -> Self {
        let mut tracker = Self {
            entries: HashMap::new(),
            config_path: config_path.to_path_buf(),
            dirty: false,
        };
        
        if config_path.exists() {
            if let Ok(data) = Self::load_from_file(config_path) {
                // Load and prune stale entries
                for entry in data.accesses {
                    if Self::calculate_score(&entry) > MIN_SCORE_THRESHOLD {
                        tracker.entries.insert(entry.path.clone(), entry);
                    }
                }
            }
        }
        
        tracker
    }
    
    fn load_from_file(path: &Path) -> Result<AccessData> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("reading file access data from {}", path.display()))?;
        toml::from_str(&contents).context("parsing file_access.toml")
    }
    
    /// Record a file access.
    pub fn record_access(&mut self, file_path: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.entries
            .entry(file_path.to_string())
            .and_modify(|e| {
                e.count = (e.count + 1.0).min(MAX_COUNT);
                e.last_accessed = now;
            })
            .or_insert(AccessEntry {
                path: file_path.to_string(),
                count: 1.0,
                last_accessed: now,
            });
        
        self.dirty = true;
    }
    
    /// Get frecency score for a file path.
    pub fn get_score(&self, file_path: &str) -> f64 {
        self.entries
            .get(file_path)
            .map(Self::calculate_score)
            .unwrap_or(0.0)
    }
    
    /// Calculate frecency score (frequency * recency decay).
    fn calculate_score(entry: &AccessEntry) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let hours_ago = (now.saturating_sub(entry.last_accessed)) / 3600;
        
        let decay = match hours_ago {
            0..=1 => 4.0,       // Last hour: 4x boost
            2..=24 => 2.0,      // Last day: 2x boost
            25..=168 => 1.0,    // Last week: normal
            169..=720 => 0.5,   // Last month: halved
            _ => 0.25,          // Older: quarter
        };
        
        entry.count * decay
    }
    
    /// Save to disk if dirty.
    pub fn save(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }
        
        // Sort by score and keep top entries
        let mut entries: Vec<_> = self.entries.values().cloned().collect();
        entries.sort_by(|a, b| {
            Self::calculate_score(b)
                .partial_cmp(&Self::calculate_score(a))
                .unwrap()
        });
        entries.truncate(MAX_ENTRIES);
        
        let data = AccessData { accesses: entries };
        
        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let contents = toml::to_string_pretty(&data)?;
        fs::write(&self.config_path, contents)?;
        
        self.dirty = false;
        Ok(())
    }
}

impl Drop for FileAccessTracker {
    fn drop(&mut self) {
        let _ = self.save();
    }
}

