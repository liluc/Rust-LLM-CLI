//! Tier 4: User feedback and learning.
//!
//! When all automated tiers fail, ask the user for clarification and learn
//! from their response to improve future classifications.

use crate::tools::TOOLS;

/// Generate user feedback prompt with tool suggestions.
pub fn generate_feedback_prompt(input: &str) -> String {
    let mut prompt = format!(
        "I'm not sure what you want to do with: \"{}\"\n\n\
         Did you mean:\n",
        input
    );
    
    // Show top tools as options (first 8 tools)
    for (idx, tool) in TOOLS.iter().take(8).enumerate() {
        prompt.push_str(&format!(
            "  [{}] {} - {}\n",
            idx + 1,
            tool.name,
            tool.description
        ));
    }
    
    prompt.push_str("\nOptions:\n");
    prompt.push_str("  • Type a number to select a tool\n");
    prompt.push_str("  • Describe what you want in plain English (I'll generate the command)\n");
    prompt.push_str("    Example: \"stage and commit only, no push\"\n");
    prompt.push_str("    Example: \"deploy to staging server\"\n");
    prompt.push_str("  • Type 'cmd: <commands>' for explicit shell commands\n");
    prompt.push_str("    Example: cmd: git add -A && git commit -m 'update'\n");
    prompt.push_str("  • Type 'none' to skip\n");
    prompt.push_str("\nI'll remember your choice for next time.");
    
    prompt
}

/// Workflow definition from inline syntax.
#[derive(Debug, Clone)]
pub struct WorkflowDefinition {
    pub name: String,
    pub steps: Vec<(String, String)>,  // (step_name, command)
    pub parameters: Vec<String>,  // Extracted {param} names
}

/// Feedback response type.
pub enum FeedbackResponse {
    ToolSelection(usize),
    ExplicitCommand(String),
    NaturalLanguageDescription(String),
    WorkflowInline(WorkflowDefinition),
    None,
    Invalid,
}

/// Parse user's feedback response.
pub fn parse_feedback_response(response: &str) -> FeedbackResponse {
    let trimmed = response.trim();
    
    // Check for "none"
    if trimmed.to_lowercase() == "none" {
        return FeedbackResponse::None;
    }
    
    // Check for workflow inline syntax (starts with "workflow:")
    if let Some(workflow_def) = parse_workflow_syntax(response) {
        return FeedbackResponse::WorkflowInline(workflow_def);
    }
    
    // Check for explicit command definition (starts with "cmd:" or legacy "macro:")
    let explicit_cmd = trimmed.strip_prefix("cmd:").or_else(|| trimmed.strip_prefix("cmd "))
        .or_else(|| trimmed.strip_prefix("macro:"))
        .or_else(|| trimmed.strip_prefix("macro "));
    
    if let Some(cmd_text) = explicit_cmd {
        let cmd = cmd_text.trim();
        if !cmd.is_empty() {
            return FeedbackResponse::ExplicitCommand(cmd.to_string());
        }
    }
    
    // Try to parse as number
    if let Ok(num) = trimmed.parse::<usize>() {
        if num > 0 && num <= TOOLS.len() {
            return FeedbackResponse::ToolSelection(num - 1); // Convert to 0-based index
        }
    }
    
    // If it's not empty and not a number, treat as natural language description
    if !trimmed.is_empty() {
        return FeedbackResponse::NaturalLanguageDescription(trimmed.to_string());
    }
    
    FeedbackResponse::Invalid
}

/// Parse workflow inline syntax.
/// 
/// Supported formats:
/// - Simple: "workflow: name = command"
/// - Multi-step:
///   "workflow: name
///    step1: command1
///    step2: command2"
/// - With parameters: "workflow: deploy to {env}"
pub fn parse_workflow_syntax(input: &str) -> Option<WorkflowDefinition> {
    let trimmed = input.trim();
    
    // Check if it starts with "workflow:"
    let workflow_prefix = "workflow:";
    if !trimmed.to_lowercase().starts_with(workflow_prefix) {
        return None;
    }
    
    let content = &trimmed[workflow_prefix.len()..].trim();
    let lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();
    
    if lines.is_empty() {
        return None;
    }
    
    // Parse first line for workflow name
    let first_line = lines[0];
    
    // Check for simple format: "name = command"
    if let Some(eq_pos) = first_line.find('=') {
        let name = first_line[..eq_pos].trim().to_string();
        let command = first_line[eq_pos + 1..].trim().to_string();
        
        if name.is_empty() || command.is_empty() {
            return None;
        }
        
        // Extract parameters from name
        let parameters = extract_parameters_from_text(&name);
        
        return Some(WorkflowDefinition {
            name,
            steps: vec![("main".to_string(), command)],
            parameters,
        });
    }
    
    // Otherwise, multi-step format
    let name = first_line.to_string();
    let mut steps = Vec::new();
    
    // Parse remaining lines as "step_name: command"
    for line in &lines[1..] {
        if let Some(colon_pos) = line.find(':') {
            let step_name = line[..colon_pos].trim().to_string();
            let command = line[colon_pos + 1..].trim().to_string();
            
            if !step_name.is_empty() && !command.is_empty() {
                steps.push((step_name, command));
            }
        }
    }
    
    if steps.is_empty() {
        return None;
    }
    
    // Extract parameters from all commands and name
    let mut parameters = extract_parameters_from_text(&name);
    for (_, command) in &steps {
        parameters.extend(extract_parameters_from_text(command));
    }
    
    // Remove duplicates
    parameters.sort();
    parameters.dedup();
    
    Some(WorkflowDefinition {
        name,
        steps,
        parameters,
    })
}

/// Extract parameter names from text containing {param} placeholders.
fn extract_parameters_from_text(text: &str) -> Vec<String> {
    use regex::Regex;
    
    let re = Regex::new(r"\{([^}]+)\}").unwrap();
    let mut params = Vec::new();
    
    for cap in re.captures_iter(text) {
        if let Some(param) = cap.get(1) {
            let param_name = param.as_str();
            // Skip LLM placeholders (uppercase with _)
            if !param_name.contains("_") || param_name.chars().any(|c| c.is_lowercase()) {
                params.push(param_name.to_string());
            }
        }
    }
    
    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_number() {
        match parse_feedback_response("1") {
            FeedbackResponse::ToolSelection(0) => {},
            _ => panic!("Expected ToolSelection(0)"),
        }
        match parse_feedback_response("5") {
            FeedbackResponse::ToolSelection(4) => {},
            _ => panic!("Expected ToolSelection(4)"),
        }
    }

    #[test]
    fn test_parse_none() {
        match parse_feedback_response("none") {
            FeedbackResponse::None => {},
            _ => panic!("Expected None"),
        }
    }

    #[test]
    fn test_parse_explicit_command() {
        match parse_feedback_response("cmd: git status") {
            FeedbackResponse::ExplicitCommand(cmd) => assert_eq!(cmd, "git status"),
            _ => panic!("Expected ExplicitCommand"),
        }
        match parse_feedback_response("cmd git add -A") {
            FeedbackResponse::ExplicitCommand(cmd) => assert_eq!(cmd, "git add -A"),
            _ => panic!("Expected ExplicitCommand"),
        }
        // Test legacy "macro:" prefix still works
        match parse_feedback_response("macro: git status") {
            FeedbackResponse::ExplicitCommand(cmd) => assert_eq!(cmd, "git status"),
            _ => panic!("Expected ExplicitCommand for legacy macro:"),
        }
    }

    #[test]
    fn test_parse_natural_language() {
        match parse_feedback_response("stage and commit only") {
            FeedbackResponse::NaturalLanguageDescription(desc) => {
                assert_eq!(desc, "stage and commit only")
            }
            _ => panic!("Expected NaturalLanguageDescription"),
        }
        match parse_feedback_response("deploy to staging") {
            FeedbackResponse::NaturalLanguageDescription(desc) => {
                assert_eq!(desc, "deploy to staging")
            }
            _ => panic!("Expected NaturalLanguageDescription"),
        }
    }

    #[test]
    fn test_parse_invalid() {
        // Invalid number (out of range) -> treated as natural language
        match parse_feedback_response("0") {
            FeedbackResponse::NaturalLanguageDescription(_) => {},
            _ => panic!("Expected NaturalLanguageDescription for '0'"),
        }
        match parse_feedback_response("999") {
            FeedbackResponse::NaturalLanguageDescription(_) => {},
            _ => panic!("Expected NaturalLanguageDescription for '999'"),
        }
        // Empty string is invalid
        match parse_feedback_response("") {
            FeedbackResponse::Invalid => {},
            _ => panic!("Expected Invalid for empty string"),
        }
    }

    #[test]
    fn test_generate_prompt() {
        let prompt = generate_feedback_prompt("do something");
        assert!(prompt.contains("I'm not sure"));
        assert!(prompt.contains("do something"));
        assert!(prompt.contains("[1]"));
        assert!(prompt.contains("cmd:"));
    }
    
    #[test]
    fn test_parse_workflow_simple() {
        let input = "workflow: deploy = docker build && docker push";
        match parse_feedback_response(input) {
            FeedbackResponse::WorkflowInline(def) => {
                assert_eq!(def.name, "deploy");
                assert_eq!(def.steps.len(), 1);
                assert_eq!(def.steps[0].0, "main");
                assert_eq!(def.steps[0].1, "docker build && docker push");
            }
            _ => panic!("Expected WorkflowInline"),
        }
    }
    
    #[test]
    fn test_parse_workflow_multistep() {
        let input = "workflow: deploy\nbuild: cargo build --release\ntest: cargo test";
        match parse_feedback_response(input) {
            FeedbackResponse::WorkflowInline(def) => {
                assert_eq!(def.name, "deploy");
                assert_eq!(def.steps.len(), 2);
                assert_eq!(def.steps[0].0, "build");
                assert_eq!(def.steps[1].0, "test");
            }
            _ => panic!("Expected WorkflowInline"),
        }
    }
    
    #[test]
    fn test_parse_workflow_with_params() {
        let input = "workflow: deploy to {env}\npush: scp app {env}.server.com:/app/";
        match parse_feedback_response(input) {
            FeedbackResponse::WorkflowInline(def) => {
                assert_eq!(def.name, "deploy to {env}");
                assert!(def.parameters.contains(&"env".to_string()));
            }
            _ => panic!("Expected WorkflowInline"),
        }
    }
}

