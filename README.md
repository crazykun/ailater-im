# fcitx5-ai-im

一个基于 Rust 开发的 fcitx5 智能输入法，支持 AI 模型预测。

## 特性

- **拼音输入**: 完整的拼音输入支持，包括音节分割
- **AI 预测**: 集成远程 AI 模型（如 0.8B 模型）或自定义模型
- **本地模型支持**: 可选的本地推理支持（使用 candle）
- **模糊匹配**: 智能模糊拼音匹配（zh/z, ch/c, sh/s 等）
- **用户词典**: 从用户输入模式中学习
- **短语预测**: 基于上下文的短语建议

## 系统要求

- Linux 操作系统
- fcitx5 输入法框架
- Rust 1.70+ (用于编译)

## 依赖

### 运行时依赖
- fcitx5
- fcitx5-libs

### 编译依赖
- Rust (cargo)
- pkg-config

## 编译安装

### 从源码编译

```bash
# 克隆仓库
git clone https://github.com/your-repo/fcitx5-ai-im.git
cd fcitx5-ai-im

# 编译
make build

# 安装到系统目录（需要 root 权限）
sudo make install

# 或者安装到用户目录
make install-user
```

### 使用 Cargo 编译

```bash
# 基本编译（仅远程模型支持）
cargo build --release

# 包含本地模型支持
cargo build --release --features local-model

# 完整功能
cargo build --release --features full
```

## 配置

### 配置文件位置

配置文件位于 `~/.config/fcitx5-ai-im/config.toml`

首次运行时会自动创建默认配置。

### AI 模型配置

#### 使用远程模型

```toml
[model]
model_type = "remote"
api_endpoint = "http://localhost:8080/v1"
model_name = "qwen-0.8b"
```

支持的 API 格式：
- OpenAI 兼容 API
- llama.cpp server
- vLLM
- 其他兼容服务

#### 使用本地模型

```toml
[model]
model_type = "local"
local_model_path = "/path/to/model"
```

需要编译时启用 `local-model` 特性。

### 输入配置

```toml
[input]
# 模糊拼音匹配
fuzzy_pinyin = true

# 候选词数量
num_candidates = 10

# 启用 AI 短语预测
enable_phrase_prediction = true
```

## 使用方法

1. 安装后重启 fcitx5：
   ```bash
   fcitx5 -r
   ```

2. 在 fcitx5 配置中添加 "AI Pinyin" 输入法

3. 切换到 AI Pinyin 输入法开始使用

### 快捷键

| 按键 | 功能 |
|------|------|
| 字母键 | 输入拼音 |
| 数字键 1-9 | 选择候选词 |
| 空格 | 选择第一个候选词 |
| 回车 | 确认输入 |
| Esc | 取消输入 |
| 退格 | 删除字符 |
| 上/下 | 翻页 |
| 左/右 | 移动光标 |

## 项目结构

```
fcitx5-ai-im/
├── src/
│   ├── lib.rs          # 库入口
│   ├── ffi.rs          # FFI 绑定
│   ├── ffi_exports.rs  # fcitx5 插件接口
│   ├── engine.rs       # 输入引擎核心
│   ├── model.rs        # AI 模型客户端
│   ├── pinyin.rs       # 拼音处理
│   ├── dictionary.rs   # 词典管理
│   └── config.rs       # 配置管理
├── include/
│   └── fcitx5.h        # fcitx5 C API 头文件
├── data/
│   ├── config.toml     # 示例配置
│   └── system.dict     # 系统词典
├── conf/
│   └── fcitx5-ai-im.conf  # fcitx5 插件配置
├── Cargo.toml          # Rust 项目配置
├── Makefile            # 构建脚本
└── README.md           # 本文件
```

## 开发

### 运行测试

```bash
make test
# 或
cargo test --all-features
```

### 代码检查

```bash
make lint
# 或
cargo clippy --all-features
```

### 生成文档

```bash
make doc
# 或
cargo doc --no-deps --all-features
```

## AI 模型集成

### 支持的模型类型

1. **远程模型** (推荐)
   - 通过 HTTP API 调用
   - 支持 OpenAI 兼容格式
   - 可使用任何部署的模型服务

2. **本地模型**
   - 使用 candle 进行本地推理
   - 支持 GGUF 格式模型
   - 需要 `local-model` 特性

### 推荐模型

- Qwen-0.8B (默认)
- TinyLlama
- 其他小型语言模型

### 部署 AI 服务

使用 llama.cpp 部署：

```bash
# 下载模型
wget https://huggingface.co/Qwen/Qwen-7B-Chat-GGUF/resolve/main/qwen7b-chat-q4_0.gguf

# 启动服务
./server -m qwen7b-chat-q4_0.gguf --host 0.0.0.0 --port 8080
```

使用 vLLM 部署：

```bash
pip install vllm
python -m vllm.entrypoints.openai.api_server --model Qwen/Qwen-7B-Chat --port 8080
```

## 故障排除

### 输入法不显示

1. 检查 fcitx5 是否正确加载插件：
   ```bash
   fcitx5-diagnose
   ```

2. 确认库文件已安装：
   ```bash
   ls /usr/lib/fcitx5/libfcitx5_ai_im.so
   ```

### AI 预测不工作

1. 检查 API 端点是否可访问：
   ```bash
   curl http://localhost:8080/v1/models
   ```

2. 查看日志：
   ```bash
   journalctl -f | grep fcitx5
   ```

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

## 致谢

- fcitx5 开发团队
- Rust 社区
- 所有贡献者
