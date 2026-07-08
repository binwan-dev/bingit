use serde::{Deserialize, Serialize};

use crate::config;

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
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

pub async fn generate_commit_message(
    diff: &str,
    prompt: &str,
    api_key: &str,
) -> anyhow::Result<String> {
    let client = reqwest::Client::new();

    let request = ChatRequest {
        model: config::MODEL.to_string(),
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

    let url = format!("{}/v1/chat/completions", config::API_BASE_URL);

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("调用 DeepSeek API 失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "DeepSeek API 返回错误 ({}): {}",
            status,
            body
        ));
    }

    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("解析 DeepSeek API 响应失败: {}", e))?;

    let message = chat_response
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| anyhow::anyhow!("DeepSeek API 返回空响应"))?;

    Ok(message)
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
                        "content": "feat: 添加新功能"
                    }
                }
            ]
        }"#;

        let response: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.content, "feat: 添加新功能");
    }

    #[test]
    fn test_chat_response_empty_choices() {
        let json = r#"{"choices": []}"#;
        let response: ChatResponse = serde_json::from_str(json).unwrap();
        assert!(response.choices.is_empty());
    }
}
