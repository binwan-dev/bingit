use std::process::Command;

pub fn is_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn get_staged_diff() -> anyhow::Result<String> {
    let output = Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .map_err(|e| anyhow::anyhow!("执行 git diff --cached 失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("git diff --cached 失败: {}", stderr));
    }

    let diff = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("解析 git diff 输出失败: {}", e))?;

    Ok(diff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_git_repo() {
        assert!(is_git_repo());
    }

    #[test]
    fn test_get_staged_diff_empty() {
        let diff = get_staged_diff().unwrap();
        assert!(diff.trim().is_empty());
    }
}
