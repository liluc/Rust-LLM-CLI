//! Workflow management and execution.
//!
//! Handles matching user input to workflows, extracting parameters,
//! and executing multi-step workflows with parameter expansion.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};

use crate::learned::Workflow;
use crate::custom_command_generator::expand_command_handlers;
use crate::config::Config;
use crate::commands::{run_shell_command, format_error};

/// Manages workflow storage and execution.
pub struct WorkflowManager {
    workflows: HashMap<String, Workflow>,
}

impl WorkflowManager {
    /// Create a new WorkflowManager with the given workflows.
    pub fn new(workflows: HashMap<String, Workflow>) -> Self {
        Self { workflows }
    }
    
    /// Get all workflows.
    pub fn get_workflows(&self) -> &HashMap<String, Workflow> {
        &self.workflows
    }
    
    /// Get a specific workflow by name (case-insensitive).
    pub fn get_workflow(&self, name: &str) -> Option<&Workflow> {
        self.workflows.get(&name.to_lowercase())
    }
    
    /// Match user input to a workflow, extracting parameter values.
    /// Returns (workflow, extracted_params) where extracted_params maps param names to values.
    pub fn match_workflow(&self, input: &str) -> Option<(&Workflow, HashMap<String, String>)> {
        let input_lower = input.to_lowercase();
        
        // First, try exact match
        if let Some(workflow) = self.workflows.get(&input_lower) {
            return Some((workflow, HashMap::new()));
        }
        
        // Then try parameter-aware matching
        for workflow in self.workflows.values() {
            if let Some(params) = Self::extract_parameters(&workflow.name, input) {
                return Some((workflow, params));
            }
        }
        
        None
    }
    
    /// Extract parameter values from user input based on workflow pattern.
    /// Example: workflow name "deploy to {env}", input "deploy to staging"
    /// Returns: {"env": "staging"}
    fn extract_parameters(pattern: &str, input: &str) -> Option<HashMap<String, String>> {
        let pattern_lower = pattern.to_lowercase();
        let input_lower = input.to_lowercase();
        
        // Split pattern and input into tokens
        let pattern_tokens: Vec<&str> = pattern_lower.split_whitespace().collect();
        let input_tokens: Vec<&str> = input_lower.split_whitespace().collect();
        
        if pattern_tokens.len() != input_tokens.len() {
            return None;
        }
        
        let mut params = HashMap::new();
        
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
    
    /// Execute a workflow with the given parameters.
    pub fn execute_workflow(
        &self,
        workflow: &Workflow,
        params: HashMap<String, String>,
        config: &Config,
        cwd: &Path,
        repo_root: Option<&Path>,
        callback: &mut dyn FnMut(&str), // Callback for progress messages
    ) -> Result<()> {
        let total_steps = workflow.steps.len();
        
        for (idx, step) in workflow.steps.iter().enumerate() {
            // Show progress
            callback(&format!(
                "[{}/{}] {}: {}",
                idx + 1,
                total_steps,
                step.name,
                step.command
            ));
            
            // Expand parameters in command
            let mut expanded = step.command.clone();
            for (param_name, param_value) in &params {
                expanded = expanded.replace(&format!("{{{}}}", param_name), param_value);
            }
            
            // Expand LLM placeholders
            let repo_path = repo_root.unwrap_or(cwd);
            expanded = expand_command_handlers(&expanded, config, repo_path)
                .with_context(|| format!("expanding placeholders in step '{}'", step.name))?;
            
            // Execute command
            match run_shell_command(cwd, &expanded) {
                Ok(output) => {
                    if !output.trim().is_empty() {
                        callback(&output);
                    }
                }
                Err(e) => {
                    let error_msg = format!("Step '{}' failed: {}", step.name, format_error(&e));
                    if !step.continue_on_error {
                        anyhow::bail!(error_msg);
                    } else {
                        callback(&format!("{} (continuing)", error_msg));
                    }
                }
            }
        }
        
        callback("✓ All steps completed!");
        Ok(())
    }
    
    /// Prompt user for missing parameters.
    /// Returns updated params map with user-provided values.
    pub fn prompt_for_parameters(
        workflow: &Workflow,
        mut params: HashMap<String, String>,
    ) -> Result<HashMap<String, String>> {
        for param in &workflow.parameters {
            if !params.contains_key(&param.name) {
                // Would need user input here - for now use default if available
                if let Some(default) = &param.default {
                    params.insert(param.name.clone(), default.clone());
                } else {
                    anyhow::bail!("Missing required parameter: {}", param.name);
                }
            }
        }
        
        Ok(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_parameters() {
        let pattern = "deploy to {env}";
        let input = "deploy to staging";
        
        let params = WorkflowManager::extract_parameters(pattern, input).unwrap();
        assert_eq!(params.get("env"), Some(&"staging".to_string()));
    }
    
    #[test]
    fn test_extract_parameters_multiple() {
        let pattern = "deploy {app} to {env}";
        let input = "deploy myapp to production";
        
        let params = WorkflowManager::extract_parameters(pattern, input).unwrap();
        assert_eq!(params.get("app"), Some(&"myapp".to_string()));
        assert_eq!(params.get("env"), Some(&"production".to_string()));
    }
    
    #[test]
    fn test_extract_parameters_no_match() {
        let pattern = "deploy to {env}";
        let input = "build the app";
        
        assert!(WorkflowManager::extract_parameters(pattern, input).is_none());
    }
}

