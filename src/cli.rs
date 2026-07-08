use std::io::{self, Write};

pub enum UserAction {
    Commit,
    Regenerate,
    Quit,
}

pub fn show_message(message: &str) {
    println!();
    println!("\x1b[1;36m══════════════════════════════════════════\x1b[0m");
    println!("\x1b[1;33m  AI 生成的提交信息:\x1b[0m");
    println!("\x1b[1;36m══════════════════════════════════════════\x1b[0m");
    println!();
    println!("\x1b[1;32m  {}\x1b[0m", message);
    println!();
    println!("\x1b[1;36m══════════════════════════════════════════\x1b[0m");
}

pub fn get_user_action() -> UserAction {
    loop {
        print!("\x1b[1;37m[回车] 提交  [r] 重新生成  [q] 退出: \x1b[0m");
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();

        match input.trim() {
            "" => return UserAction::Commit,
            "r" | "R" => return UserAction::Regenerate,
            "q" | "Q" => return UserAction::Quit,
            _ => {
                println!("\x1b[1;31m无效输入，请重新选择\x1b[0m");
            }
        }
    }
}
