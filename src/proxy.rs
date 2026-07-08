use std::process::{exit, Command};

pub fn proxy(args: &[String]) -> ! {
    let mut cmd = Command::new("git");
    cmd.args(&args[1..]);

    let status = cmd
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .unwrap_or_else(|e| {
            eprintln!("无法执行 git: {}", e);
            exit(1);
        });

    exit(status.code().unwrap_or(1));
}
