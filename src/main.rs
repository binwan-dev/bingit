mod ai;
mod cli;
mod config;
mod git;
mod proxy;

use std::process::exit;

fn is_ai_commit(args: &[String]) -> bool {
    args.len() >= 3 && args[1] == "ai" && args[2] == "commit"
}

async fn run_ai_commit() {
    let api_key = match config::load_api_key() {
        Ok(k) => k,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };

    let prompt = config::load_prompt();

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
        let message = match ai::generate_commit_message(&diff, &prompt, &api_key).await {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        };

        cli::show_message(&message);

        match cli::get_user_action() {
            cli::UserAction::Commit => {
                let status = std::process::Command::new("git")
                    .args(["commit", "-m", &message])
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

    if is_ai_commit(&args) {
        run_ai_commit().await;
    } else {
        proxy::proxy(&args);
    }
}
