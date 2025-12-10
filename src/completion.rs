//! Autocompletion provider for the CLI.
//!
//! Provides ghost-text completion with smart path detection.

use std::path::Path;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::{learned::LearnedAliases, tools::TOOLS};

pub struct CompletionProvider {
    tool_examples: Vec<String>,
    matcher: SkimMatcherV2,
}

impl CompletionProvider {
    pub fn new() -> Self {
        // Collect all tool examples
        let mut tool_examples = Vec::new();
        for tool in TOOLS {
            for example in tool.examples {
                tool_examples.push(example.to_string());
            }
        }
        
        Self {
            tool_examples,
            matcher: SkimMatcherV2::default(),
        }
    }
    
    /// Get the best completion for ghost text display.
    /// Returns the suffix to append after the current input.
    pub fn get_ghost_completion(
        &self,
        input: &str,
        cwd: &Path,
        history: &[String],
        learned: &LearnedAliases,
    ) -> Option<String> {
        if input.is_empty() {
            return None;
        }
        
        // Extract last word
        let last_word = input.split_whitespace().last().unwrap_or(input);
        
        // Check if last word is path-like
        if is_path_like(last_word, cwd) {
            if let Some(path_completion) = complete_path(last_word, cwd) {
                // Return only the suffix beyond the last word
                if path_completion.len() > last_word.len() {
                    return Some(path_completion[last_word.len()..].to_string());
                }
            }
        }
        
        // Full input fuzzy matching
        let best_match = self.get_best_match(input, history, learned)?;
        
        // Return only the suffix
        if best_match.len() > input.len() && best_match.starts_with(input) {
            Some(best_match[input.len()..].to_string())
        } else {
            None
        }
    }
    
    fn get_best_match(
        &self,
        prefix: &str,
        history: &[String],
        learned: &LearnedAliases,
    ) -> Option<String> {
        let mut best_score = 0;
        let mut best_match: Option<String> = None;
        
        // Check tool examples
        for example in &self.tool_examples {
            if let Some(score) = self.matcher.fuzzy_match(example, prefix) {
                if score > best_score {
                    best_score = score;
                    best_match = Some(example.clone());
                }
            }
        }
        
        // Check history
        for entry in history.iter().rev().take(50) {
            let display = entry
                .trim()
                .strip_prefix('$')
                .or_else(|| entry.trim().strip_prefix('!'))
                .map(|s| s.trim())
                .unwrap_or(entry);
            
            if let Some(score) = self.matcher.fuzzy_match(display, prefix) {
                if score > best_score {
                    best_score = score;
                    best_match = Some(display.to_string());
                }
            }
        }
        
        // Check learned aliases
        for alias in learned.get_all_phrases() {
            if let Some(score) = self.matcher.fuzzy_match(&alias, prefix) {
                if score > best_score {
                    best_score = score;
                    best_match = Some(alias);
                }
            }
        }
        
        best_match
    }
}

/// Check if a word looks like a path.
fn is_path_like(word: &str, cwd: &Path) -> bool {
    // Contains path separator
    if word.contains('/') || word.contains('\\') {
        return true;
    }
    
    // Starts with special path prefixes
    if word.starts_with('.') || word.starts_with('~') {
        return true;
    }
    
    // Matches existing file or directory in cwd
    if cwd.join(word).exists() {
        return true;
    }
    
    false
}

/// Complete a path by listing matching files/folders.
fn complete_path(partial: &str, cwd: &Path) -> Option<String> {
    // Handle relative paths
    let path_str = if partial.starts_with('~') {
        // Expand home directory
        if let Some(home) = dirs::home_dir() {
            partial.replacen('~', &home.to_string_lossy(), 1)
        } else {
            return None;
        }
    } else {
        partial.to_string()
    };
    
    let path = Path::new(&path_str);
    
    // Check if the partial path itself is a complete directory
    let resolved_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    
    if resolved_path.is_dir() && !path_str.ends_with('/') {
        // Complete directory by adding trailing slash
        return Some(format!("{}/", partial));
    }
    
    // Split into directory and filename prefix
    let (dir, prefix) = if path_str.ends_with('/') || path_str.ends_with('\\') {
        // Path ends with slash - list contents of this directory
        // Remove trailing slash for proper path handling
        let trimmed = path_str.trim_end_matches(&['/', '\\'][..]);
        (Path::new(trimmed), "")
    } else if let Some(parent) = path.parent() {
        (parent, path.file_name()?.to_str()?)
    } else {
        (Path::new("."), path_str.as_str())
    };
    
    // Resolve directory relative to cwd
    let search_dir = if dir.is_absolute() {
        dir.to_path_buf()
    } else {
        cwd.join(dir)
    };
    
    if !search_dir.exists() {
        return None;
    }
    
    // Find first matching entry
    if let Ok(entries) = std::fs::read_dir(search_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with(prefix) {
                    // Build full completion relative to the input path
                    let completion = if path_str.contains('/') || path_str.contains('\\') {
                        // Reconstruct the full path
                        dir.join(name).to_string_lossy().to_string()
                    } else {
                        name.to_string()
                    };
                    
                    // Add trailing slash for directories
                    if entry.path().is_dir() {
                        return Some(format!("{}/", completion));
                    } else {
                        return Some(completion);
                    }
                }
            }
        }
    }
    
    None
}

impl Default for CompletionProvider {
    fn default() -> Self {
        Self::new()
    }
}

