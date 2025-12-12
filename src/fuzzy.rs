//! Tier 1: Fuzzy matching for intent resolution.
//!
//! This tier provides instant matching against:
//! - Learned aliases (highest priority)
//! - Exact tool names
//! - Exact tool examples
//! - Fuzzy matches against tool names (using FZF algorithm)

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use crate::{
    intent::ParsedIntent,
    learned::LearnedAliases,
    tools::TOOLS,
};

// Minimum score for FZF matching (scores are typically in range 0-200+)
// Adjust this value based on testing - higher = stricter matching
// Note: Good typo corrections like "staus" -> "status" score ~103
//       Weak matches like "hello" in "shell command" score ~86
const FUZZY_MIN_SCORE: i64 = 90;

/// Try exact and fuzzy matching against tool names, examples, learned aliases, and workflows.
/// Returns Some(intent) if a confident match is found, None otherwise.
pub fn fuzzy_match(input: &str, learned: &LearnedAliases) -> Option<ParsedIntent> {
    let input_lower = input.to_lowercase();
    
    // 1. Check learned aliases first (highest priority)
    if let Some(intent) = learned.match_phrase(&input_lower) {
        return Some(intent);
    }
    
    // 2. Check workflows (with parameter extraction)
    // This returns a special intent marker that will be handled in app.rs
    if let Some(workflow) = learned.get_workflow(&input_lower) {
        // Exact match - no parameters
        let mut intent = ParsedIntent::new("execute_workflow", 1.0);
        intent.args.query = Some(workflow.name.clone());
        return Some(intent);
    }
    
    // Try parameter-aware workflow matching
    for (_workflow_name, workflow) in learned.get_workflows() {
        if let Some(_params) = extract_workflow_parameters(&workflow.name, input) {
            let mut intent = ParsedIntent::new("execute_workflow", 0.95);
            intent.args.query = Some(workflow.name.clone());
            return Some(intent);
        }
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
    
    // 4. Fuzzy match against tool names using FZF/Skim algorithm
    let matcher = SkimMatcherV2::default();
    let mut best_match: Option<(&str, i64)> = None;
    
    for tool in TOOLS {
        if let Some(score) = matcher.fuzzy_match(tool.name, &input_lower) {
            if score >= FUZZY_MIN_SCORE {
                match best_match {
                    Some((_, best_score)) if score > best_score => {
                        best_match = Some((tool.name, score));
                    }
                    None => {
                        best_match = Some((tool.name, score));
                    }
                    _ => {}
                }
            }
        }
    }
    
    // 5. Fuzzy match against tool examples (if no tool name matched)
    if best_match.is_none() {
        for tool in TOOLS {
            for example in tool.examples {
                if let Some(score) = matcher.fuzzy_match(&example.to_lowercase(), &input_lower) {
                    // Slightly lower confidence for example matches
                    let adjusted_score = (score as f64 * 0.9) as i64;
                    if adjusted_score >= FUZZY_MIN_SCORE {
                        match best_match {
                            Some((_, best_score)) if adjusted_score > best_score => {
                                best_match = Some((tool.name, adjusted_score));
                            }
                            None => {
                                best_match = Some((tool.name, adjusted_score));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    
    // Convert score to normalized confidence (0.85-1.0 range)
    best_match.map(|(tool, score)| {
        // Normalize score to confidence. FZF scores are typically 0-200+
        // We map scores to 0.85-1.0 range for consistency with other matchers
        let confidence = ((score.min(200) as f32 / 200.0) * 0.15 + 0.85).min(1.0);
        ParsedIntent::new(tool, confidence)
    })
}

/// Extract parameter values from user input based on workflow pattern.
/// Example: workflow name "deploy to {env}", input "deploy to staging"
/// Returns: Some({"env": "staging"})
fn extract_workflow_parameters(pattern: &str, input: &str) -> Option<std::collections::HashMap<String, String>> {
    let pattern_lower = pattern.to_lowercase();
    let input_lower = input.to_lowercase();
    
    // Split pattern and input into tokens
    let pattern_tokens: Vec<&str> = pattern_lower.split_whitespace().collect();
    let input_tokens: Vec<&str> = input_lower.split_whitespace().collect();
    
    if pattern_tokens.len() != input_tokens.len() {
        return None;
    }
    
    let mut params = std::collections::HashMap::new();
    
    for (p_token, i_token) in pattern_tokens.iter().zip(input_tokens.iter()) {
        if p_token.starts_with('{') && p_token.ends_with('}') {
            // Extract parameter name
            let param_name = &p_token[1..p_token.len()-1];
            params.insert(param_name.to_string(), i_token.to_string());
        } else if p_token != i_token {
            // Non-parameter tokens must match exactly
            return None;
        }
    }
    
    Some(params)
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
        // Test with an actual example from the save_work tool
        let intent = fuzzy_match("push my changes", &learned).unwrap();
        assert_eq!(intent.tool, "save_work");
        assert_eq!(intent.confidence, 0.95);
    }

    #[test]
    fn test_fuzzy_match() {
        let learned = LearnedAliases::default();
        // FZF requires characters in sequential order, so "staus" (s-t-a-u-s matches s-t-a-t-u-s)
        let intent = fuzzy_match("staus", &learned).unwrap();
        assert_eq!(intent.tool, "status");
        assert!(intent.confidence >= 0.85);
    }

    #[test]
    fn test_fuzzy_no_false_positive() {
        let learned = LearnedAliases::default();
        // "hello" should NOT match "shell" - this was the original bug
        // With FZF matcher and threshold of 90, weak matches are rejected
        let intent = fuzzy_match("hello", &learned);
        assert!(intent.is_none(), "hello should not match shell or any other tool");
    }

    #[test]
    fn test_no_match() {
        let learned = LearnedAliases::default();
        let intent = fuzzy_match("xyz123abc", &learned);
        assert!(intent.is_none());
    }
    
    #[test]
    fn test_extract_workflow_parameters() {
        let pattern = "deploy to {env}";
        let input = "deploy to staging";
        
        let params = extract_workflow_parameters(pattern, input).unwrap();
        assert_eq!(params.get("env"), Some(&"staging".to_string()));
    }
    
    #[test]
    fn test_extract_workflow_parameters_no_match() {
        let pattern = "deploy to {env}";
        let input = "build the app";
        
        assert!(extract_workflow_parameters(pattern, input).is_none());
    }
}

