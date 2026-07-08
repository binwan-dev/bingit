# bingit

git 透明代理 + AI 生成 Conventional Commits 提交信息。

## 安装

```bash
cargo install --git https://github.com/binwan-dev/bingit.git
```

或本地编译：

```bash
git clone git@github.com:binwan-dev/bingit.git
cd bingit
cargo build --release
cp target/release/bingit /usr/local/bin/
```

## 配置

### API Key

设置环境变量：

```bash
export BINGIT_AI_KEY="sk-xxx"
```

### 自定义 Prompt（可选）

创建 `~/.config/bingit/prompt.md` 写入自定义系统提示词，不创建则使用内置默认提示词。

## 使用

### 透明代理

`bingit` 直接替代 `git`，所有命令透传：

```bash
bingit status
bingit add .
bingit log --oneline
```

### AI 生成提交信息

```bash
git add .
bingit ai commit
```

AI 会根据已暂存的 diff 生成 Conventional Commits 格式的提交信息，展示后供你选择：

- **回车** — 直接提交
- **r** — 重新生成
- **q** — 退出不提交

## 工作原理

```
bingit ai commit
  → 读取 BINGIT_AI_KEY 和 prompt
  → git diff --cached 获取暂存变更
  → 调用 DeepSeek API 生成提交信息
  → 交互确认
  → git commit -m <message>
```

## 依赖

- DeepSeek API Key（当前模型：`deepseek-v4-flash`）
- Rust 1.85+
