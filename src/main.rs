mod ai;
mod cli;
mod config;
mod git;
mod proxy;
mod token_logging;

use std::process::exit;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn is_ai_commit(args: &[String]) -> bool {
    args.len() >= 3 && args[1] == "ai" && args[2] == "commit"
}

fn is_version_flag(args: &[String]) -> bool {
    args.len() == 2 && (args[1] == "--version" || args[1] == "-V")
}

async fn run_ai_commit() {
    let cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };

    if let Err(e) = config::validate_config(&cfg) {
        eprintln!("{}", e);
        exit(1);
    }

    let prompt = config::load_prompt(&cfg);

    if !git::is_git_repo() {
        eprintln!("当前目录不是 git 仓库");
        exit(1);
    }

    let diff = match git::get_staged_diff() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };

    if diff.trim().is_empty() {
        eprintln!("没有已暂存的更改");
        exit(0);
    }

    loop {
        let result = match ai::generate_commit_message(&diff, &prompt, &cfg).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        };

        if let Some(ref usage) = result.usage {
            token_logging::log_token_usage(&cfg, &cfg.ai_commit.model, usage);
        }

        cli::show_message(&result.message);

        match cli::get_user_action() {
            cli::UserAction::Commit => {
                let status = std::process::Command::new("git")
                    .args(["commit", "-m", &result.message])
                    .stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .status()
                    .unwrap_or_else(|e| {
                        eprintln!("执行 git commit 失败: {}", e);
                        exit(1);
                    });

                exit(status.code().unwrap_or(1));
            }
            cli::UserAction::Regenerate => continue,
            cli::UserAction::Quit => {
                println!("已取消");
                exit(0);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if is_version_flag(&args) {
        println!("bingit v{}", VERSION);
        exit(0);
    }

    if is_ai_commit(&args) {
        run_ai_commit().await;
    } else {
        proxy::proxy(&args);
    }
}