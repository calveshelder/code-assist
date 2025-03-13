use crate::config::Config;
use anyhow::{anyhow, Context, Result};
use log::debug;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: usize,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    content: String,
}

pub struct LlmClient {
    client: Client,
    config: Config,
}

impl LlmClient {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::new();

        Ok(Self {
            client,
            config: config.clone(),
        })
    }

    pub async fn process_command(&self, command: &str, context: &str) -> Result<String> {
        let system_message = format!(
            "You are CodeAssist, an AI coding assistant that helps users with their codebase. \
            You analyze the context and the user's command, and respond with specific actions to take. \
            Respond in JSON format with the following structure: \
            {{\"action\": \"<action_type>\", \"details\": {{...action specific details...}}}}. \
            Possible actions: edit_file, answer_question, execute_command, git_operation."
        );

        let user_message = format!(
            "Command: {}\n\nContext from codebase:\n{}",
            command, context
        );

        let request = ChatRequest {
            model: self.config.llm.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_message,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_message,
                },
            ],
            temperature: self.config.llm.temperature,
            max_tokens: self.config.llm.max_tokens,
        };

        debug!("Sending request to LLM: {:?}", request);

        let url = format!("{}/chat/completions", self.config.llm.api_url);
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!("Bearer {}", self.config.llm.api_key),
            )
            .json(&request)
            .send()
            .await
            .context("Failed to send request to LLM API")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            return Err(anyhow!("LLM API error: {} - {}", status, text));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse LLM API response")?;

        if chat_response.choices.is_empty() {
            return Err(anyhow!("LLM returned empty response"));
        }

        Ok(chat_response.choices[0].message.content.clone())
    }
}

