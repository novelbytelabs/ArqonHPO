use anyhow::{Result, Context};
use reqwest::blocking::Client;
use serde_json::json;
use std::env;

/// Trait for LLM clients used in healing
pub trait LlmClient {
    fn generate_fix(&mut self, prompt: &str) -> Result<String>;
}

/// HTTP-based LLM client compatible with OpenAI API (or Ollama)
pub struct RemoteLlm {
    client: Client,
    base_url: String,
    model: String,
    api_key: String,
}

impl RemoteLlm {
    pub fn new() -> Result<Self> {
        let base_url = env::var("ARQON_LLM_URL").unwrap_or_else(|_| "http://localhost:11434/v1".to_string());
        let model = env::var("ARQON_LLM_MODEL").unwrap_or_else(|_| "deepseek-coder:1.3b".to_string());
        let api_key = env::var("ARQON_LLM_KEY").unwrap_or_else(|_| "ollama".to_string());

        Ok(Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
            api_key,
        })
    }
}

impl LlmClient for RemoteLlm {
    fn generate_fix(&mut self, prompt: &str) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);
        
        let body = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": "You are an expert software engineer. Output only the fixed code block."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.2
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .context("Failed to send request to LLM provider")?;

        if !response.status().is_success() {
            let error = response.text()?;
            return Err(anyhow::anyhow!("LLM request failed: {}", error));
        }

        let json: serde_json::Value = response.json()?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

        // Simple cleanup of code fences if present
        let clean_content = content
            .lines()
            .filter(|l| !l.trim().starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(clean_content)
    }
}

