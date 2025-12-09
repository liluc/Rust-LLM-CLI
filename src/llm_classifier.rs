//! Tier 3: Small LLM classifier fallback.
//!
//! Uses a small, fast local LLM (like qwen2:0.5b) to classify intent when
//! deterministic methods fail. This provides flexibility for novel phrasing.

use anyhow::{Context, Result};

use crate::{intent::ParsedIntent, tools::TOOLS};

const LLM_CONFIDENCE_THRESHOLD: f32 = 0.5;

/// Use a small LLM to classify intent when other methods fail.
/// Returns Some(intent) if LLM provides a valid tool match, None otherwise.
/// Returns "chat" intent if user is just having a conversation.
pub async fn classify_with_llm(input: &str, model: &str) -> Result<Option<ParsedIntent>> {
    // Build tool list for prompt
    let tool_names: Vec<_> = TOOLS.iter().map(|t| t.name).collect();
    let tools_str = tool_names.join(", ");
    
    let prompt = format!(
        "You are a command classifier. The user is either:\n\
         1. Trying to perform an ACTION with the codebase/git (use one of the tools)\n\
         2. Just CHATTING or asking questions about concepts (respond with 'chat')\n\
         \n\
         Available tools: {}\n\
         \n\
         Rules:\n\
         - If asking about concepts/explanations/how things work → 'chat'\n\
         - If asking to DO something (run, execute, show, save, commit) → tool name\n\
         - When in doubt, prefer 'chat'\n\
         \n\
         Examples:\n\
         Input: \"hello\" → chat\n\
         Input: \"how are you?\" → chat\n\
         Input: \"what is rust?\" → chat\n\
         Input: \"explain how this works\" → chat\n\
         Input: \"can you explain the code?\" → chat\n\
         Input: \"show me the git status\" → status\n\
         Input: \"save my work\" → save_work\n\
         Input: \"run the tests\" → run_tests\n\
         \n\
         Respond with ONLY the tool name or 'chat', nothing else.\n\
         \n\
         User input: \"{}\"\n\
         Response:",
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
    
    // Check for "chat" response
    if llm_response == "chat" || llm_response.contains("chat") {
        return Ok(Some(ParsedIntent::new("chat", LLM_CONFIDENCE_THRESHOLD)));
    }
    
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
    
    // If LLM couldn't classify, return None (will fall through to ask_user)
    Ok(None)
}

