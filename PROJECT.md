# ailater-im 项目概览

## 项目简介

ailater-im 是一个使用 Rust 开发的 Linux 输入法，专为 fcitx5 框架设计。它结合了传统拼音输入和现代 AI 技术，提供智能的中文输入体验。

## 核心特性

### 1. 拼音输入引擎
- 完整的拼音音节解析
- 智能音节分割
- 支持全拼和简拼
- 模糊拼音匹配（zh/z, ch/c, sh/s, an/ang, en/eng, in/ing 等）

### 2. AI 模型集成
- **远程模型支持**: 通过 HTTP API 调用远程 AI 模型
- **本地模型支持**: 可选的本地推理（需要 candle 库）
- **混合模式**: 远程优先，本地备用
- 支持多种模型格式和 API

### 3. 词典系统
- 系统词典：预置常用词汇
- 用户词典：学习用户输入习惯
- 频率管理：动态调整候选词顺序
- 持久化存储：保存用户偏好

### 4. 配置系统
- TOML 格式配置文件
- 热重载支持
- 细粒度配置选项

## 技术架构

```
┌─────────────────────────────────────────────────────────────┐
│                     fcitx5 输入法框架                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    FFI 导出层 (ffi_exports.rs)               │
│  - fcitx_im_create()    - fcitx_im_destroy()                │
│  - fcitx_im_key_event() - fcitx_im_reset()                  │
│  - fcitx_im_focus_in()  - fcitx_im_focus_out()              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    输入引擎 (engine.rs)                       │
│  - 键盘事件处理                                              │
│  - 候选词管理                                                │
│  - 输入状态维护                                              │
└─────────────────────────────────────────────────────────────┘
         │              │              │              │
         ▼              ▼              ▼              ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ 拼音解析器    │ │ 词典管理器    │ │ AI 模型客户端 │ │ 配置管理器    │
│ (pinyin.rs)  │ │(dictionary.rs)│ │  (model.rs)  │ │ (config.rs)  │
└──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘
```

## 模块说明

### src/lib.rs
库入口文件，定义公共 API 和模块导出。

### src/ffi.rs
定义 fcitx5 C API 的 Rust 绑定，包括：
- KeySym: 键码枚举
- KeyState: 修饰键状态
- IMReturnValue: 返回值类型
- FcitxIMClass: 输入法类结构

### src/ffi_exports.rs
导出给 fcitx5 调用的 C 函数：
- fcitx_im_create(): 创建输入法实例
- fcitx_im_destroy(): 销毁实例
- fcitx_im_key_event(): 处理键盘事件
- fcitx_im_reset(): 重置输入状态
- fcitx_im_focus_in/out(): 焦点处理

### src/engine.rs
核心输入引擎：
- InputState: 输入状态管理
- InputEngine: 主引擎结构
- 候选词生成和排序
- 键盘事件分发

### src/pinyin.rs
拼音处理模块：
- PinyinParser: 拼音解析器
- FuzzyPinyinMatcher: 模糊匹配器
- 内置拼音-汉字映射表

### src/dictionary.rs
词典管理：
- Dictionary: 词典管理器
- DictEntry: 词条结构
- 频率更新和持久化

### src/model.rs
AI 模型客户端：
- ModelBackend: 模型后端 trait
- RemoteModelClient: 远程 API 客户端
- HybridModelClient: 混合客户端
- 预测结果缓存

### src/config.rs
配置管理：
- Config: 主配置结构
- ModelConfig: 模型配置
- InputConfig: 输入配置
- UIConfig: 界面配置
- DictionaryConfig: 词典配置

## 构建和安装

### 依赖要求
- Rust 1.70+
- fcitx5 开发库
- pkg-config

### 构建步骤

```bash
# 安装 Rust（如果未安装）
./install-rust.sh

# 构建
make build

# 安装到系统
sudo make install

# 或安装到用户目录
make install-user
```

## 配置 AI 模型

### 使用 llama.cpp

```bash
# 下载模型
wget https://huggingface.co/Qwen/Qwen-0.5B-Chat-GGUF/resolve/main/qwen-0.5b-chat-q4_0.gguf

# 启动服务
llama-server -m qwen-0.5b-chat-q4_0.gguf --host 0.0.0.0 --port 8080
```

### 配置文件

编辑 `~/.config/ailater-im/config.toml`:

```toml
[model]
model_type = "remote"
api_endpoint = "http://localhost:8080/v1"
model_name = "qwen"
```

## 扩展开发

### 添加新的模型后端

1. 实现 ModelBackend trait
2. 在 model.rs 中添加新结构
3. 更新 create_model_client 函数

### 添加新的输入模式

1. 在 engine.rs 中添加新的状态处理
2. 更新 handle_key 方法
3. 添加相应的候选词生成逻辑

## 性能优化

- 使用 parking_lot 替代标准库锁
- 候选词缓存
- 异步模型调用
- 延迟加载词典

## 已知限制

1. 本地模型需要大量内存
2. 首次启动加载词典较慢
3. 某些生僻字可能不在词典中

## 未来计划

- [ ] 支持更多拼音方案（双拼等）
- [ ] 云端词典同步
- [ ] 更多 AI 模型支持
- [ ] 表情符号输入
- [ ] 语音输入集成
