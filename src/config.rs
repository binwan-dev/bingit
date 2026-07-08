pub const API_BASE_URL: &str = "https://api.deepseek.com";
pub const MODEL: &str = "deepseek-v4-flash";

const DEFAULT_PROMPT: &str = r#"你是一个专业的软件工程师。根据提供的 git diff 内容，生成一条符合 Conventional Commits 规范的提交信息。
要求：
1. 类型准确（feat/fix/docs/refactor/test/chore 等）
2. 描述简洁清晰、用中文
3. 只输出提交信息本身，不要任何解释"#;

pub fn load_api_key() -> anyhow::Result<String> {
    std::env::var("BINGIT_AI_KEY")
        .map_err(|_| anyhow::anyhow!("请设置环境变量 BINGIT_AI_KEY"))
}

pub fn load_prompt() -> String {
    let config_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".config")
        .join("bingit")
        .join("prompt.md");

    std::fs::read_to_string(&config_path).unwrap_or_else(|_| DEFAULT_PROMPT.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_prompt_returns_default_when_file_missing() {
        let prompt = load_prompt();
        assert!(prompt.contains("Conventional Commits"));
        assert!(!prompt.is_empty());
    }

    #[test]
    fn test_load_api_key_missing() {
        unsafe { std::env::remove_var("BINGIT_AI_KEY"); }
        let result = load_api_key();
        assert!(result.is_err());
    }

    #[test]
    fn test_load_api_key_present() {
        unsafe { std::env::set_var("BINGIT_AI_KEY", "sk-test-key"); }
        let result = load_api_key();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "sk-test-key");
        unsafe { std::env::remove_var("BINGIT_AI_KEY"); }
    }
}
