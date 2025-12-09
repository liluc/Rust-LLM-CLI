//! Tier 3: Small LLM classifier fallback.
//!
//! Uses a small, fast local LLM (like qwen2:0.5b) to classify intent when
//! deterministic methods fail. This provides flexibility for novel phrasing.

use anyhow::{Context, Result};

use crate::{intent::ParsedIntent, tools::TOOLS};

const LLM_CONFIDENCE_THRESHOLD: f32 = 0.5;

/// Use a small LLM to classify intent when other methods fail.
/// Returns Some(intent) if LLM provides a valid tool match, None otherwise.
pub async fn classify_with_llm(input: &str, model: &str) -> Result<Option<ParsedIntent>> {
    // Build tool list for prompt
    let tool_names: Vec<_> = TOOLS.iter().map(|t| t.name).collect();
    let tools_str = tool_names.join(", ");
    
    let prompt = format!(
        "You are a command classifier. Respond with ONLY the tool name, nothing else.\n\
         Available tools: {}\n\n\
         User input: \"{}\"\n\
         Tool name:",
        tools_str, input
    );
    
    // Call Ollama
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.1,
                "num_predict": 20,
            }
        }))
        .send()
        .await
        .context("calling Ollama API for LLM classification")?;
    
    if !response.status().is_success() {
        return Ok(None);
    }
    
    let result: serde_json::Value = response.json().await.context("parsing LLM response")?;
    let llm_response = result["response"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_lowercase();
    
    // Check if response matches a valid tool
    for tool in TOOLS {
        if llm_response == tool.name || llm_response.contains(tool.name) {
            return Ok(Some(ParsedIntent::new(tool.name, LLM_CONFIDENCE_THRESHOLD)));
        }
    }
    
    // Also check if tool name is contained in response (handles "the tool is: status")
    for tool in TOOLS {
        if tool.name.contains(&llm_response) && llm_response.len() > 3 {
            return Ok(Some(ParsedIntent::new(tool.name, LLM_CONFIDENCE_THRESHOLD * 0.9)));
        }
    }
    
    Ok(None)
}

