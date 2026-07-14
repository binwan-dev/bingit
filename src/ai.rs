use serde::{Deserialize, Serialize};

use crate::config::{Config, ProviderConfig};

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

#[derive(Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub struct CommitResult {
    pub message: String,
    pub usage: Option<Usage>,
}

pub async fn generate_commit_message(
    diff: &str,
    prompt: &str,
    config: &Config,
    provider: &ProviderConfig,
) -> anyhow::Result<CommitResult> {
    let (_, model_name) = config.ai_commit.model.split_once('/').unwrap();

    let client = reqwest::Client::new();

    let request = ChatRequest {
        model: model_name.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: format!("以下是 git diff 内容：\n\n{}", diff),
            },
        ],
    };

    let url = format!("{}/v1/chat/completions", provider.api_base_url);

    let api_key = crate::config::resolve_api_key(provider)?;

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("调用 API ({}) 失败: {}", provider.api_base_url, e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "API ({}) 返回错误 ({}): {}",
            provider.api_base_url,
            status,
            body
        ));
    }

    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("解析 API 响应失败: {}", e))?;

    let message = chat_response
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| anyhow::anyhow!("API 返回空响应"))?;

    Ok(CommitResult {
        message,
        usage: chat_response.usage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            model: "deepseek-v4-flash".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "test prompt".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: "test diff".to_string(),
                },
            ],
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("deepseek-v4-flash"));
        assert!(json.contains("system"));
        assert!(json.contains("test prompt"));
    }

    #[test]
    fn test_chat_response_deserialization() {
        let json = r#"{
            "choices": [
                {
                    "message": {
                        "content": "feat: \u6dfb\u52a0\u65b0\u529f\u80fd"
                    }
                }
            ],
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 10,
                "total_tokens": 110
            }
        }"#;

        let response: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.content, "feat: 添加新功能");
        let usage = response.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 10);
        assert_eq!(usage.total_tokens, 110);
    }

    #[test]
    fn test_chat_response_empty_choices() {
        let json = r#"{"choices": [], "usage": null}"#;
        let response: ChatResponse = serde_json::from_str(json).unwrap();
        assert!(response.choices.is_empty());
        assert!(response.usage.is_none());
    }
}