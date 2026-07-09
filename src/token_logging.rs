use std::path::PathBuf;

use crate::ai::Usage;
use crate::config::Config;

pub fn log_token_usage(config: &Config, model: &str, usage: &Usage) {
    if !config.token_logging {
        return;
    }

    let log_path = PathBuf::from(&config.log_path);
    if let Some(parent) = log_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("创建日志目录 {:?} 失败: {}", parent, e);
            return;
        }
    }

    let entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "model": model,
        "input_tokens": usage.prompt_tokens,
        "output_tokens": usage.completion_tokens,
        "total_tokens": usage.total_tokens,
    });

    let line = match serde_json::to_string(&entry) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("序列化日志条目失败: {}", e);
            return;
        }
    };

    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(mut file) => {
            use std::io::Write;
            if let Err(e) = writeln!(file, "{}", line) {
                eprintln!("写入日志文件 {:?} 失败: {}", log_path, e);
            }
        }
        Err(e) => {
            eprintln!("打开日志文件 {:?} 失败: {}", log_path, e);
        }
    }
}