//! Tier 1: Fuzzy matching for intent resolution.
//!
//! This tier provides instant matching against:
//! - Learned aliases (highest priority)
//! - Exact tool names
//! - Exact tool examples
//! - Fuzzy matches against tool names

use strsim::jaro_winkler;

use crate::{
    intent::ParsedIntent,
    learned::LearnedAliases,
    tools::TOOLS,
};

const FUZZY_THRESHOLD: f64 = 0.85;

/// Try exact and fuzzy matching against tool names, examples, and learned aliases.
/// Returns Some(intent) if a confident match is found, None otherwise.
pub fn fuzzy_match(input: &str, learned: &LearnedAliases) -> Option<ParsedIntent> {
    let input_lower = input.to_lowercase();
    
    // 1. Check learned aliases first (highest priority)
    if let Some(intent) = learned.match_phrase(&input_lower) {
        return Some(intent);
    }
    
    // 2. Exact match against tool names
    for tool in TOOLS {
        if input_lower == tool.name {
            return Some(ParsedIntent::new(tool.name, 1.0));
        }
    }
    
    // 3. Exact match against tool examples
    for tool in TOOLS {
        for example in tool.examples {
            if input_lower == example.to_lowercase() {
                return Some(ParsedIntent::new(tool.name, 0.95));
            }
        }
    }
    
    // 4. Fuzzy match against tool names
    let mut best_match: Option<(&str, f64)> = None;
    
    for tool in TOOLS {
        let similarity = jaro_winkler(&input_lower, tool.name);
        if similarity >= FUZZY_THRESHOLD {
            match best_match {
                Some((_, best_score)) if similarity > best_score => {
                    best_match = Some((tool.name, similarity));
                }
                None => {
                    best_match = Some((tool.name, similarity));
                }
                _ => {}
            }
        }
    }
    
    // 5. Fuzzy match against tool examples (lower confidence)
    if best_match.is_none() {
        for tool in TOOLS {
            for example in tool.examples {
                let similarity = jaro_winkler(&input_lower, &example.to_lowercase());
                if similarity >= FUZZY_THRESHOLD + 0.05 {
                    match best_match {
                        Some((_, best_score)) if similarity > best_score => {
                            best_match = Some((tool.name, similarity * 0.9));
                        }
                        None => {
                            best_match = Some((tool.name, similarity * 0.9));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    best_match.map(|(tool, score)| ParsedIntent::new(tool, score as f32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_tool_name() {
        let learned = LearnedAliases::default();
        let intent = fuzzy_match("status", &learned).unwrap();
        assert_eq!(intent.tool, "status");
        assert_eq!(intent.confidence, 1.0);
    }

    #[test]
    fn test_exact_example() {
        let learned = LearnedAliases::default();
        let intent = fuzzy_match("save work", &learned).unwrap();
        assert_eq!(intent.tool, "save_work");
        assert_eq!(intent.confidence, 0.95);
    }

    #[test]
    fn test_fuzzy_match() {
        let learned = LearnedAliases::default();
        let intent = fuzzy_match("stauts", &learned).unwrap();
        assert_eq!(intent.tool, "status");
        assert!(intent.confidence >= 0.85);
    }

    #[test]
    fn test_no_match() {
        let learned = LearnedAliases::default();
        let intent = fuzzy_match("xyz123abc", &learned);
        assert!(intent.is_none());
    }
}

