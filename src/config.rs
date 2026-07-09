use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Clone)]
pub struct ProviderConfig {
    pub name: String,
    #[serde(rename = "ApiKey")]
    pub api_key: String,
    #[serde(rename = "ApiBaseUrl")]
    pub api_base_url: String,
    pub models: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AiCommitConfig {
    #[serde(rename = "PromptPath")]
    pub prompt_path: String,
    #[serde(rename = "Model")]
    pub model: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(rename = "AICommit")]
    pub ai_commit: AiCommitConfig,
    pub provider: ProviderConfig,
    #[serde(rename = "TokenLogging")]
    pub token_logging: bool,
    #[serde(rename = "LogPath")]
    pub log_path: String,
}

fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".config")
        .join("bingit")
}

fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

fn default_log_path() -> String {
    let home = dirs::home_dir().unwrap_or_default();
    if cfg!(target_os = "macos") {
        home.join("Library")
            .join("Logs")
            .join("bingit")
            .join("ai.log")
            .to_string_lossy()
            .to_string()
    } else if cfg!(target_os = "windows") {
        dirs::data_local_dir()
            .unwrap_or_default()
            .join("bingit")
            .join("ai.log")
            .to_string_lossy()
            .to_string()
    } else {
        home.join(".local")
            .join("share")
            .join("bingit")
            .join("ai.log")
            .to_string_lossy()
            .to_string()
    }
}

fn default_prompt_path() -> String {
    config_dir()
        .join("prompt.md")
        .to_string_lossy()
        .to_string()
}

fn default_config() -> Config {
    Config {
        ai_commit: AiCommitConfig {
            prompt_path: default_prompt_path(),
            model: "deepseek/deepseek-v4-flash".to_string(),
        },
        provider: ProviderConfig {
            name: "deepseek".to_string(),
            api_key: String::new(),
            api_base_url: "https://api.deepseek.com".to_string(),
            models: vec!["deepseek-v4-flash".to_string()],
        },
        token_logging: false,
        log_path: default_log_path(),
    }
}

pub fn load_config() -> anyhow::Result<Config> {
    let path = config_path();

    if !path.exists() {
        let config = default_config();
        let dir = config_dir();
        std::fs::create_dir_all(&dir).map_err(|e| {
            anyhow::anyhow!("无法创建配置目录 {:?}: {}", dir, e)
        })?;
        let content = serde_json::to_string_pretty(&config).map_err(|e| {
            anyhow::anyhow!("序列化默认配置失败: {}", e)
        })?;
        std::fs::write(&path, content)
            .map_err(|e| anyhow::anyhow!("写入默认配置到 {:?} 失败: {}", path, e))?;
        return Ok(config);
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("读取配置文件 {:?} 失败: {}", path, e))?;

    let config: Config = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("解析配置文件 {:?} 失败: {}", path, e))?;

    Ok(config)
}

pub fn validate_config(config: &Config) -> anyhow::Result<()> {
    let (provider_name, model_name) = config.ai_commit.model.split_once('/').ok_or_else(|| {
        anyhow::anyhow!(
            "AICommit.Model 格式错误: {}，应为 provider_name/model_name",
            config.ai_commit.model
        )
    })?;

    if provider_name.is_empty() || model_name.is_empty() {
        return Err(anyhow::anyhow!(
            "AICommit.Model 格式错误: {}，应为 provider_name/model_name",
            config.ai_commit.model
        ));
    }

    if config.provider.name != provider_name {
        return Err(anyhow::anyhow!(
            "AICommit.Model 中的 provider '{}' 与配置的 provider.name '{}' 不匹配",
            provider_name,
            config.provider.name
        ));
    }

    if !config.provider.models.contains(&model_name.to_string()) {
        return Err(anyhow::anyhow!(
            "模型 '{}' 不在 provider '{}' 的 models 列表中",
            model_name,
            provider_name
        ));
    }

    Ok(())
}

pub fn resolve_api_key(config: &Config) -> anyhow::Result<String> {
    if !config.provider.api_key.is_empty() {
        return Ok(config.provider.api_key.clone());
    }
    std::env::var("BINGIT_AI_KEY")
        .map_err(|_| anyhow::anyhow!("请设置 BINGIT_AI_KEY 环境变量或在配置文件中填写 provider.ApiKey"))
}

pub fn load_prompt(config: &Config) -> String {
    let path = PathBuf::from(&config.ai_commit.prompt_path);
    std::fs::read_to_string(&path).unwrap_or_else(|_| {
        "你是一个专业的软件工程师。根据提供的 git diff 内容，生成一条符合 Conventional Commits 规范的提交信息。\n\
         要求：\n\
         1. 类型准确（feat/fix/docs/refactor/test/chore 等）\n\
         2. 描述简洁清晰、用中文\n\
         3. 只输出提交信息本身，不要任何解释"
            .to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_valid_model_path() {
        let config = default_config();
        let (provider_name, model_name) = config.ai_commit.model.split_once('/').unwrap();
        assert_eq!(provider_name, "deepseek");
        assert_eq!(model_name, "deepseek-v4-flash");
    }

    #[test]
    fn test_validate_config_valid() {
        let config = default_config();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_bad_model_format() {
        let mut config = default_config();
        config.ai_commit.model = "invalid".to_string();
        let err = validate_config(&config).unwrap_err();
        assert!(err.to_string().contains("格式错误"));
    }

    #[test]
    fn test_validate_config_provider_mismatch() {
        let mut config = default_config();
        config.ai_commit.model = "openai/gpt-4".to_string();
        let err = validate_config(&config).unwrap_err();
        assert!(err.to_string().contains("不匹配"));
    }

    #[test]
    fn test_validate_config_model_not_in_list() {
        let mut config = default_config();
        config.ai_commit.model = "deepseek/deepseek-v4-pro".to_string();
        let err = validate_config(&config).unwrap_err();
        assert!(err.to_string().contains("不在"));
    }

    #[test]
    fn test_resolve_api_key() {
        let config = default_config();

        unsafe { std::env::remove_var("BINGIT_AI_KEY"); }
        assert!(resolve_api_key(&config).is_err());

        unsafe { std::env::set_var("BINGIT_AI_KEY", "sk-from-env"); }
        assert_eq!(resolve_api_key(&config).unwrap(), "sk-from-env");
        unsafe { std::env::remove_var("BINGIT_AI_KEY"); }

        let mut config = default_config();
        config.provider.api_key = "sk-from-config".to_string();
        assert_eq!(resolve_api_key(&config).unwrap(), "sk-from-config");
    }

    #[test]
    fn test_load_prompt_returns_default_when_file_missing() {
        let config = default_config();
        let prompt = load_prompt(&config);
        assert!(prompt.contains("Conventional Commits"));
        assert!(!prompt.is_empty());
    }
}