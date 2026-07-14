# bingit

git 透明代理 + AI 生成 Conventional Commits 提交信息。

## 安装

### Linux / macOS

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

### Windows

1. 安装 Rust 工具链：访问 https://rustup.rs 下载并运行 `rustup-init.exe`
2. 克隆并编译：

```powershell
git clone git@github.com:binwan-dev/bingit.git
cd bingit
cargo build --release
```

3. 将编译产物添加到 PATH：

```powershell
# 复制到自定义目录（如 C:\tools）
mkdir C:\tools
copy target\release\bingit.exe C:\tools\

# 将 C:\tools 添加到系统环境变量 PATH 中
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\tools", "User")
```

## 配置

### 模型 Provider

默认配置支持以下 provider，位于 `~/.config/bingit/config.json`：

| Provider | API Base URL | 可用模型 |
|----------|-------------|---------|
| `deepseek` | `https://api.deepseek.com` | `deepseek-v4-flash` |
| `volcengine` | `https://ark.cn-beijing.volces.com/api/v3` | `deepseek-v4-pro`, `deepseek-v4-flash` |

默认使用 DeepSeek。切换 provider 只需修改配置文件中的 `AICommit.Model` 字段：

```json
{
  "AICommit": {
    "Model": "volcengine/deepseek-v4-pro"
  }
}
```

也可以手动添加其他 OpenAI 兼容的 provider。

### API Key

设置环境变量：

```bash
# Linux / macOS
export BINGIT_AI_KEY="sk-xxx"

# Windows PowerShell
$env:BINGIT_AI_KEY="sk-xxx"
```

或在配置文件中填写 `ApiKey` 字段。

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

### 重命名为 git 实现完全透明代理

将 `bingit` 重命名为 `git` 并放入 PATH 中比系统 `git` 更靠前的位置，即可完全无感代理：

**Linux / macOS：**

```bash
# 将 bingit 重命名为 git 放到 PATH 前面
sudo cp target/release/bingit /usr/local/bin/git

# 确保 /usr/local/bin 在 /usr/bin 之前
echo $PATH | grep /usr/local/bin
```

**Windows：**

```powershell
# 将 bingit.exe 重命名为 git.exe 放到 PATH 前面
copy target\release\bingit.exe C:\tools\git.exe

# 在系统环境变量中确保 C:\tools 在 Git 安装目录之前
```

重命名后直接使用 `git` 命令即可，`git ai commit` 会触发 AI 提交，其余命令正常透传。

## 工作原理

```
bingit ai commit
  → 读取 BINGIT_AI_KEY 和 prompt
  → 根据 AICommit.Model 匹配对应的 provider 配置
  → git diff --cached 获取暂存变更
  → 调用对应 provider 的 API 生成提交信息
  → 交互确认
  → git commit -m <message>
```

## 依赖

- 支持的 AI Provider API Key（DeepSeek / 火山引擎 / 其他 OpenAI 兼容服务）
- Rust 1.85+