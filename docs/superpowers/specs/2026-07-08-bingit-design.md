# Bingit Design Spec

## Overview

`bingit` 是一个 Rust CLI 工具，作为 `git` 的透明代理，额外提供 `ai commit` 子命令，利用 DeepSeek AI 分析 `git diff --cached` 生成 Conventional Commits 规范的提交信息，并交互式确认后提交。

## CLI Routing

```
bingit ai commit           → AI 生成提交信息并提交
bingit ai commit --help    → 帮助
bingit <any other args>    → 透传给 git <args>
```

## Module Architecture

```
src/
├── main.rs          # CLI 路由：解析参数，分发到 proxy 或 ai 流程
├── proxy.rs         # 透传执行 git 命令
├── git.rs           # git diff --cached 获取已暂存变更
├── ai.rs            # DeepSeek API 调用
├── config.rs        # 环境变量 BINGIT_AI_KEY + prompt 文件读取
└── cli.rs           # AI 交互界面：展示消息、等待用户选择
```

## Module Details

### main.rs

- 解析命令行参数
- 若 `args[1] == "ai" && args[2] == "commit"` → 进入 AI 提交流程
- 否则 → `args[0]` 替换为 `git`，透传所有参数

### proxy.rs

- `pub fn proxy(args: &[String])` → `std::process::Command::new("git").args(&args[1..]).status()`
- 将 stdin/stdout/stderr 透传给子进程
- 用 git 的退出码作为自身退出码

### config.rs

- `BINGIT_AI_KEY` 环境变量读取（不存在则报错退出）
- `~/.config/bingit/prompt.md` 读取（不存在则用内置默认 prompt）
- 默认 prompt：`你是一个专业的软件工程师。根据提供的 git diff 内容，生成一条符合 Conventional Commits 规范的提交信息。要求：1. 类型准确（feat/fix/docs/refactor/test/chore 等）；2. 描述简洁清晰、用中文；3. 只输出提交信息本身，不要任何解释。`
- API endpoint 常量：`https://api.deepseek.com`
- 模型常量：`deepseek-v4-flash`

### git.rs

- `pub fn is_git_repo() -> bool` — 检查当前目录是否在 git 仓库内
- `pub fn get_staged_diff() -> Result<String>` — 执行 `git diff --cached` 返回 diff 内容
- 若不在 git 仓库内或无暂存内容 → 打印错误并退出

### ai.rs

- `pub async fn generate_commit_message(diff: &str, prompt: &str, api_key: &str) -> Result<String>`
- 调用 DeepSeek Chat Completions API (`/v1/chat/completions`)
- system message: prompt 文件内容
- user message: `以下是 git diff 内容：\n\n{diff}`
- 模型：`deepseek-v4-flash`
- 超时：60s
- 解析 response 中的 `choices[0].message.content`

### cli.rs

- `pub async fn interactive_commit(message: &str) -> Result<()>`
- 打印分隔线和 AI 生成的消息
- 提示 `[回车] 提交  [r] 重新生成  [q] 退出`
- 用户按回车 → 执行 `git commit -m <message>`
- 用户按 `r` → 返回信号让调用方重新生成
- 用户按 `q` → 退出

## Data Flow

```
main.rs
  → config::load_prompt()          // 读取 prompt
  → config::load_api_key()         // 读取 API key
  → git::is_git_repo()             // 检查 git 仓库
  → git::get_staged_diff()         // 获取 diff
  → loop:
      → ai::generate_commit_message()  // 调用 DeepSeek
      → cli::interactive_commit()      // 展示 + 交互
      → 若回车 → git commit -m → 退出
      → 若 r   → continue loop
      → 若 q   → 退出
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1 | 异步运行时 |
| reqwest | 0.12 | HTTPS 客户端 (json feature) |
| serde | 1 | 序列化 derive |
| serde_json | 1 | JSON 处理 |
| anyhow | 1 | 错误处理 |
| dirs | 5 | 获取用户 home 目录 |

## Error Handling

- `BINGIT_AI_KEY` 未设置 → 打印 "请设置环境变量 BINGIT_AI_KEY" 并退出
- 不在 git 仓库 → 打印 "当前目录不是 git 仓库" 并退出
- 无暂存内容 → 打印 "没有已暂存的更改" 并退出
- AI API 调用失败 → 打印错误信息，提示用户重试或退出
- `git commit` 失败 → 打印 git 的错误输出

## Testing Strategy

- 单元测试：`config` 模块的 prompt 加载逻辑
- 集成测试：通过 mock HTTP server 测试 `ai` 模块的请求构造和响应解析
- 手动验证：在真实仓库中 `git add` 后运行 `bingit ai commit`
