# ailater-im

<div align="center">

![AI Later](src/img/icons/scalable/apps/ailater-im.svg)

**AI 智能输入法 | fcitx5 中文输入方案**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![fcitx5](https://img.shields.io/badge/fcitx5-5.1+-green.svg)](https://fcitx-im.org/)

[特性](#特性) • [快速开始](#快速开始) • [配置](#配置) • [开发](#开发)

</div>

---

## 简介

**ailater-im** 是一个基于 Rust 开发的 fcitx5 智能输入法插件，融合传统拼音输入与现代 AI 技术，提供流畅的中文输入体验。

- **输入法名称**: AI Later
- **唯一标识**: `ai-later`
- **默认模式**: 本地词典模式（AI 默认禁用）
- **架构**: Cargo Workspace（主插件 + 独立配置工具）

## 特性

### 核心功能

| 功能 | 说明 |
|------|------|
| 🎯 **全拼输入** | 完整的拼音音节解析与智能分割 |
| ⚡ **简拼支持** | 拼音首字母简写输入（如 `zh` → `中国`） |
| 🔍 **模糊匹配** | 智能模糊拼音（zh/z, ch/c, sh/s, an/ang 等） |
| 📚 **双词典** | 系统词典 + 用户学习词典 |
| 🤖 **AI 预测** | 可选的 AI 模型集成（远程/本地/混合） |
| 🎨 **图形配置** | 独立配置工具 `ailater-config` |

### AI 模型支持

| 模式 | 说明 |
|------|------|
| **none** | 禁用 AI，仅使用本地词典（默认） |
| **remote** | 通过 HTTP API 调用远程 AI 服务 |
| **local** | 使用 candle 进行本地模型推理 |
| **hybrid** | 远程优先，失败时回退到本地 |

## 快速开始

### 系统要求

- Linux 操作系统
- fcitx5 输入法框架 (5.1+)
- Rust 1.70+ (编译时)
- pkg-config

### 安装

#### 方式一：从源码安装

```bash
# 克隆仓库
git clone https://github.com/crazykun/ailater-im.git
cd ailater-im

# 编译（workspace 包含主插件和配置工具）
make build

# 安装到系统目录（需要 root）
sudo make install

# 或安装到用户目录
make install-user
```

#### 方式二：使用 Cargo Workspace

```bash
# 构建所有 members
cargo build --workspace --release

# 只构建主插件
cargo build -p ailater-im --release

# 只构建配置工具
cargo build -p ailater-config --release
```

### 启用输入法

1. 重启 fcitx5：
   ```bash
   fcitx5 -r
   ```

2. 在 fcitx5 配置中添加 **"AI Later"** 输入法

3. 切换到 AI Later 开始使用

### 快捷键

| 按键 | 功能 |
|------|------|
| `a`-`z` | 输入拼音 |
| `1`-`9` | 选择候选词 |
| `空格` | 选择第一个候选词 |
| `回车` | 确认当前输入 |
| `Esc` | 取消输入 |
| `退格` | 删除字符 |
| `上/下` | 候选词翻页 |
| `左/右` | 移动光标 |
| `PageUp/Down` | 快速翻页 |

## 配置

### 配置方式

ailater-im 提供两种配置方式：

#### 方式一：图形化配置工具（推荐）

```bash
# 启动配置界面
ailater-config

# 或从系统菜单启动
# 设置 -> AI Later 输入法设置
```

**功能特点：**
- 分标签页管理（AI 模型、输入设置、界面、词典）
- 实时配置预览和验证
- 一键保存和重置
- 支持打开配置目录

#### 方式二：手动编辑配置文件

配置文件位于：`~/.config/ailater-im/config.toml`

### AI 模型配置

#### 禁用 AI（默认）

```toml
[model]
model_type = "none"
```

#### 使用远程模型

```toml
[model]
model_type = "remote"
api_endpoint = "http://localhost:8080/v1"
model_name = "qwen-0.8b"

[input]
enable_phrase_prediction = true
```

#### 使用本地模型

```toml
[model]
model_type = "local"
local_model_path = "/path/to/model.gguf"
```

#### 混合模式

```toml
[model]
model_type = "hybrid"
api_endpoint = "http://localhost:8080/v1"
local_model_path = "/path/to/model.gguf"
```

### 拼音输入配置

```toml
[input]
# 模糊拼音匹配
fuzzy_pinyin = true

# 候选词数量
num_candidates = 30

# 智能纠错
smart_correction = true

# 标点符号自动上屏
auto_commit_on_punctuation = true
```

### 词典配置

```toml
[dictionary]
# 启用用户词典学习
enable_learning = true

# 用户词典最大条目数
max_user_dictionary_size = 100000

# 频率衰减因子（遗忘旧词条）
frequency_decay = 0.99
```

### 完整配置示例

```toml
[model]
model_type = "remote"           # none, remote, local, hybrid
api_endpoint = "http://localhost:8080/v1"
model_name = "qwen-0.8b"
max_tokens = 50
temperature = 0.7
enable_cache = true

[input]
max_preedit_length = 64
num_candidates = 10
fuzzy_pinyin = true
smart_correction = true
page_size = 5
enable_phrase_prediction = false
min_ai_input_length = 2

[ui]
show_candidate_numbers = true
vertical_candidate_list = false
font_size = 12

[dictionary]
enable_learning = true
max_user_dictionary_size = 100000
frequency_decay = 0.99
```

## AI 服务部署

### 使用 llama.cpp

```bash
# 下载模型
wget https://huggingface.co/Qwen/Qwen-0.5B-Chat-GGUF/resolve/main/qwen-0.5b-chat-q4_0.gguf

# 启动服务
llama-server -m qwen-0.5b-chat-q4_0.gguf --host 0.0.0.0 --port 8080
```

### 使用 vLLM

```bash
pip install vllm
python -m vllm.entrypoints.openai.api_server \
  --model Qwen/Qwen-0.5B-Chat \
  --port 8080
```

### 使用 ollama

```bash
# 拉取模型
ollama pull qwen:0.5b

# 启动服务
ollama serve

# 配置端点
api_endpoint = "http://localhost:11434/v1"
```

## 项目架构

### Workspace 结构

项目采用 Cargo Workspace 架构，实现逻辑分离与构建统一：

```
ailater-im/                      # Workspace 根目录
├── Cargo.toml                   # Workspace 定义 + 主库配置
├── Cargo.lock                   # 统一依赖锁文件
│
├── src/                         # 主库 (ailater-im)
│   ├── lib.rs                   # 库入口，导出配置类型
│   ├── engine.rs                # 输入引擎核心
│   ├── model.rs                 # AI 模型客户端
│   ├── pinyin.rs                # 拼音处理
│   ├── dictionary.rs            # 词典管理
│   ├── config.rs                # 配置结构定义（共享）
│   ├── ffi.rs                   # FFI 类型绑定
│   └── ffi_exports.rs           # fcitx5 插件接口
│
└── config-tool/                 # 配置工具 (ailater-config)
    ├── Cargo.toml               # 引用主库配置类型
    └── src/main.rs              # GUI 实现（egui/eframe）
```

### 设计理念

**为什么使用 Workspace？**

1. **依赖隔离** - GUI 依赖不影响输入法插件
   - 主插件被 fcitx5 加载到每个应用程序进程
   - GUI 框架依赖只在配置工具中需要

2. **代码共享** - 配置结构统一定义
   - `config.rs` 定义配置类型
   - 主插件和配置工具都引用同一结构
   - 避免重复和不一致

3. **构建统一** - 一次命令构建所有
   ```bash
   cargo build --workspace      # 构建所有
   cargo test --workspace       # 测试所有
   cargo fmt --all              # 格式化所有
   ```

4. **版本同步** - workspace 统一管理版本
   ```toml
   [workspace.package]
   version = "0.1.0"
   ```

### 架构图

```
┌─────────────────────────────────────────────────────────────┐
│                   Cargo Workspace                           │
│                                                              │
│  ┌──────────────────┐         ┌──────────────────┐          │
│  │  ailater-im      │         │  ailater-config  │          │
│  │  (cdylib 库)     │         │  (独立二进制)     │          │
│  │                  │         │                  │          │
│  │  - 输入引擎      │         │  - GUI 界面       │          │
│  │  - FFI 接口      │◄────────┤  - 引用配置类型   │          │
│  │  - 配置结构定义   │ 共享    │  - egui/eframe   │          │
│  └──────────────────┘         └──────────────────┘          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     fcitx5 输入法框架                        │
└─────────────────────────────────────────────────────────────┘
```

### 完整目录结构

```
ailater-im/
├── src/
│   ├── lib.rs              # 库入口，导出配置类型
│   ├── engine.rs           # 输入引擎核心
│   ├── model.rs            # AI 模型客户端
│   ├── pinyin.rs           # 拼音处理
│   ├── dictionary.rs       # 词典管理
│   ├── config.rs           # 配置结构定义
│   ├── ffi.rs              # FFI 类型绑定
│   ├── ffi_exports.rs      # fcitx5 插件接口
│   └── bin/
│       └── test_im.rs      # 测试工具
│
├── config-tool/            # 配置工具（独立二进制）
│   ├── src/main.rs         # GUI 实现
│   ├── Cargo.toml          # 依赖定义
│   └── icon.png            # 应用图标
│
├── include/
│   └── fcitx5.h            # fcitx5 C API 头文件
│
├── data/
│   ├── config.toml         # 示例配置
│   └── system.dict         # 系统词典
│
├── conf/
│   ├── ailater-im.conf     # fcitx5 插件配置
│   └── inputmethod/
│       └── ailater-im.conf # 输入法配置
│
├── src/img/
│   └── icons/              # 图标资源
│
├── Cargo.toml              # Workspace + 主库配置
├── Makefile                # 构建脚本
├── CLAUDE.md               # 项目开发指南
└── README.md               # 本文件
```

### 模块依赖关系

```
fcitx5
   │
   ▼
ffi_exports.rs ─────┐
                   │
                   ▼
engine.rs ───► pinyin.rs
   │              │
   │              ▼
   │         dictionary.rs
   │
   ├──► model.rs (AI)
   │
   └──► config.rs (配置)
                   ▲
                   │ shared
          config-tool/src/main.rs
```

## 开发

### Workspace 命令

```bash
# 构建所有 members
cargo build --workspace

# 构建特定 member
cargo build -p ailater-im
cargo build -p ailater-config

# 测试所有
cargo test --workspace

# 格式化所有
cargo fmt --all

# 检查所有
cargo check --workspace
cargo clippy --workspace
```

### Makefile 命令

```bash
# 构建
make build              # 构建主插件
make config-tool        # 构建配置工具
make                    # 构建所有

# 安装/卸载
make install            # 系统安装
make install-user       # 用户安装
make uninstall          # 系统卸载
make uninstall-user     # 用户卸载

# 测试
make test               # 运行所有测试
make lint               # 代码检查
make fmt                # 代码格式化
make doc                # 生成文档

# 其他
make clean              # 清理构建
make dist               # 创建发布包
make help               # 显示帮助
```

### 运行测试

```bash
# 所有测试
cargo test --workspace

# 主库测试
cargo test -p ailater-im

# 单个模块测试
cargo test -p ailater-im pinyin
```

### 代码检查

```bash
# Clippy 检查（所有）
cargo clippy --workspace -- -D warnings

# 只检查主库
cargo clippy -p ailater-im -- -D warnings

# 格式检查
cargo fmt --all -- --check
```

### 添加词典词条

词典格式：`word\tpinyin\tfrequency\tlast_used`

```
你好	nihao	1000	1704067200
世界	shijie	950	1704067200
```

## 文件安装位置

| 文件类型 | 系统安装路径 | 用户安装路径 |
|---------|-------------|-------------|
| 共享库 | `/usr/lib/x86_64-linux-gnu/fcitx5/libailater_im.so` | `~/.local/lib/x86_64-linux-gnu/fcitx5/` |
| 配置工具 | `/usr/bin/ailater-config` | `~/.local/bin/ailater-config` |
| 桌面入口 | `/usr/share/applications/ailater-config.desktop` | `~/.local/share/applications/` |
| 系统词典 | `/usr/share/ailater-im/dict/system.dict` | `~/.local/share/ailater-im/dict/` |
| 插件配置 | `/usr/share/fcitx5/addon/ailater-im.conf` | `~/.local/share/fcitx5/addon/` |
| 输入法配置 | `/usr/share/fcitx5/inputmethod/ailater-im.conf` | `~/.local/share/fcitx5/inputmethod/` |
| 应用图标 | `/usr/share/icons/hicolor/*/apps/org.fcitx.Fcitx5.ailater-im.*` | `~/.local/share/icons/hicolor/` |
| 用户词典 | `~/.local/share/ailater-im/user.dict` | 同左 |
| 配置文件 | `~/.config/ailater-im/config.toml` | 同左 |

## 故障排除

### 输入法不显示

1. 检查 fcitx5 状态：
   ```bash
   fcitx5-diagnose
   ```

2. 查看日志：
   ```bash
   journalctl -f | grep fcitx5
   ```

3. 确认库文件存在：
   ```bash
   ls /usr/lib/x86_64-linux-gnu/fcitx5/libailater_im.so
   # 或用户安装
   ls ~/.local/lib/x86_64-linux-gnu/fcitx5/libailater_im.so
   ```

### AI 预测不工作

1. 检查配置文件中 `model_type` 是否为 `remote`、`local` 或 `hybrid`

2. 测试 API 端点：
   ```bash
   curl http://localhost:8080/v1/models
   ```

3. 确认 `enable_phrase_prediction` 已启用

### 配置工具无法启动

1. 检查是否已安装：
   ```bash
   which ailater-config
   ```

2. 尝试直接运行查看错误：
   ```bash
   ailater-config
   ```

3. 或手动编辑配置文件作为替代方案

## Cargo Features

| Feature | 说明 | 默认 |
|---------|------|-----|
| `fcitx5` | fcitx5 插件支持 | ✓ |
| `remote-model` | 远程 AI 模型 | ✓ |
| `local-model` | 本地模型推理（需 candle） | - |
| `full` | 所有功能 | - |

## 技术栈

### 主插件 (ailater-im)
- **语言**: Rust 2021 Edition
- **FFI**: libc, fcitx5 C API
- **异步**: tokio
- **HTTP**: reqwest
- **序列化**: serde, toml
- **拼音**: pinyin crate
- **并发**: parking_lot, dashmap
- **本地推理**: candle (可选)

### 配置工具 (ailater-config)
- **GUI 框架**: eframe + egui
- **文件对话框**: rfd
- **打开目录**: open
- **配置类型**: 共享主库定义

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

## 致谢

- [fcitx5](https://github.com/fcitx/fcitx5) - 优秀的输入法框架
- [Rust 社区](https://www.rust-lang.org/) - 强大的语言生态
- [egui](https://github.com/emilk/egui) - 跨平台 GUI 框架
- [candle](https://github.com/huggingface/candle) - 轻量级推理框架

---

<div align="center">

**Made with ❤️ by the AI Later community**

[GitHub](https://github.com/crazykun/ailater-im) • [Issue Tracker](https://github.com/crazykun/ailater-im/issues)

</div>
