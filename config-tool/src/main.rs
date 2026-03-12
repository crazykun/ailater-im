#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

/// AI Later 配置工具
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 500.0])
            .with_min_inner_size([500.0, 400.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../icon.png")[..])
                    .unwrap_or_default(),
            )
            .with_title("AI Later 输入法配置"),
        ..Default::default()
    };

    eframe::run_native(
        "AI Later 配置",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Box::new(ConfigApp::default())
        }),
    )
}

fn setup_fonts(ctx: &egui::Context) {
    // 尝试加载中文字体
    let mut fonts = egui::FontDefinitions::default();

    // 尝试常见的中文字体路径
    let font_paths = [
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/System/Library/Fonts/PingFang.ttc", // macOS
    ];

    for path in font_paths {
        if std::path::Path::new(path).exists() {
            if let Ok(font_data) = std::fs::read(path) {
                fonts.font_data.insert(
                    "cjk".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "cjk".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("cjk".to_owned());
                break;
            }
        }
    }

    ctx.set_fonts(fonts);
}

/// 配置应用状态
struct ConfigApp {
    config: AppConfig,
    config_path: PathBuf,
    status_message: String,
    status_type: StatusType,
    selected_tab: ConfigTab,
}

#[derive(Default, PartialEq)]
enum ConfigTab {
    #[default]
    Model,
    Input,
    UI,
    Dictionary,
    About,
}

#[derive(Default, PartialEq)]
enum StatusType {
    #[default]
    None,
    Success,
    Error,
    Info,
}

impl ConfigApp {
    fn default() -> Self {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ailater-im")
            .join("config.toml");

        let config = if config_path.exists() {
            Self::load_config(&config_path).unwrap_or_default()
        } else {
            AppConfig::default()
        };

        Self {
            config,
            config_path,
            status_message: "配置已加载".to_string(),
            status_type: StatusType::Info,
            selected_tab: ConfigTab::Model,
        }
    }

    fn load_config(path: &PathBuf) -> Option<AppConfig> {
        let content = fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }

    fn save_config(&mut self) {
        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            if fs::create_dir_all(parent).is_err() {
                self.show_status("创建配置目录失败", StatusType::Error);
                return;
            }
        }

        // 序列化配置
        let toml_str = match toml::to_string_pretty(&self.config) {
            Ok(s) => s,
            Err(e) => {
                self.show_status(&format!("序列化配置失败: {}", e), StatusType::Error);
                return;
            }
        };

        // 写入文件
        match fs::write(&self.config_path, toml_str) {
            Ok(_) => {
                self.show_status("配置已保存", StatusType::Success);
            }
            Err(e) => {
                self.show_status(&format!("保存配置失败: {}", e), StatusType::Error);
            }
        }
    }

    fn show_status(&mut self, msg: &str, status: StatusType) {
        self.status_message = msg.to_string();
        self.status_type = status;
    }

    fn reload_config(&mut self) {
        self.config = Self::load_config(&self.config_path).unwrap_or_default();
        self.show_status("配置已重新加载", StatusType::Info);
    }
}

impl eframe::App for ConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("保存配置").clicked() {
                        self.save_config();
                        ui.close_menu();
                    }
                    if ui.button("重新加载").clicked() {
                        self.reload_config();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("退出").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("帮助", |ui| {
                    if ui.button("关于").clicked() {
                        self.selected_tab = ConfigTab::About;
                        ui.close_menu();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let icon = match self.status_type {
                    StatusType::Success => "✓",
                    StatusType::Error => "✗",
                    StatusType::Info => "ℹ",
                    StatusType::None => "",
                };
                ui.label(format!("{} {}", icon, self.status_message));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, ConfigTab::Model, "🤖 AI 模型");
                ui.selectable_value(&mut self.selected_tab, ConfigTab::Input, "⌨️ 输入设置");
                ui.selectable_value(&mut self.selected_tab, ConfigTab::UI, "🎨 界面");
                ui.selectable_value(&mut self.selected_tab, ConfigTab::Dictionary, "📖 词典");
                ui.selectable_value(&mut self.selected_tab, ConfigTab::About, "ℹ️ 关于");
            });

            ui.separator();

            match self.selected_tab {
                ConfigTab::Model => self.show_model_tab(ui),
                ConfigTab::Input => self.show_input_tab(ui),
                ConfigTab::UI => self.show_ui_tab(ui),
                ConfigTab::Dictionary => self.show_dictionary_tab(ui),
                ConfigTab::About => self.show_about_tab(ui),
            }

            ui.separator();

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("💾 保存配置").clicked() {
                        self.save_config();
                    }
                    if ui.button("🔄 重置为默认").clicked() {
                        self.config = AppConfig::default();
                        self.show_status("已重置为默认配置", StatusType::Info);
                    }
                });
            });
        });
    }
}

impl ConfigApp {
    fn show_model_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("AI 模型设置");
        ui.add_space(10.0);

        egui::Grid::new("model_grid").num_columns(2).spacing([10.0, 8.0]).show(ui, |ui| {
            ui.label("模型类型:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.config.model.model_type, "none".to_string(), "禁用 (仅本地词典)");
                ui.radio_value(&mut self.config.model.model_type, "remote".to_string(), "远程 API");
                ui.radio_value(&mut self.config.model.model_type, "local".to_string(), "本地模型");
            });
            ui.end_row();

            if self.config.model.model_type != "none" {
                ui.label("API 端点:");
                ui.text_edit_singleline(&mut self.config.model.api_endpoint);
                ui.end_row();

                ui.label("模型名称:");
                ui.text_edit_singleline(&mut self.config.model.model_name);
                ui.end_row();

                ui.label("最大 Token 数:");
                ui.add(egui::Slider::new(&mut self.config.model.max_tokens, 10..=500));
                ui.end_row();

                ui.label("温度 (0.0 - 2.0):");
                ui.add(egui::Slider::new(&mut self.config.model.temperature, 0.0..=2.0));
                ui.end_row();

                ui.label("启用缓存:");
                ui.checkbox(&mut self.config.model.enable_cache, "");
                ui.end_row();

                if self.config.model.enable_cache {
                    ui.label("缓存大小:");
                    ui.add(egui::Slider::new(&mut self.config.model.cache_size, 100..=50000));
                    ui.end_row();
                }
            }
        });

        ui.add_space(10.0);
        ui.collapsing("说明", |ui| {
            ui.label("• 禁用模式: 仅使用本地词典，不调用 AI");
            ui.label("• 远程 API: 调用 HTTP 接口的 AI 服务");
            ui.label("• 本地模型: 使用本地加载的 AI 模型");
        });
    }

    fn show_input_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("输入设置");
        ui.add_space(10.0);

        egui::Grid::new("input_grid").num_columns(2).spacing([10.0, 8.0]).show(ui, |ui| {
            ui.label("最大输入长度:");
            ui.add(egui::Slider::new(&mut self.config.input.max_preedit_length, 10..=100));
            ui.end_row();

            ui.label("候选词数量:");
            ui.add(egui::Slider::new(&mut self.config.input.num_candidates, 5..=20));
            ui.end_row();

            ui.label("每页候选词数:");
            ui.add(egui::Slider::new(&mut self.config.input.page_size, 3..=10));
            ui.end_row();

            ui.label("模糊拼音:");
            ui.checkbox(&mut self.config.input.fuzzy_pinyin, "支持 zh/z、ch/c、sh/s 等混淆");
            ui.end_row();

            ui.label("智能纠错:");
            ui.checkbox(&mut self.config.input.smart_correction, "");
            ui.end_row();

            ui.label("标点自动上屏:");
            ui.checkbox(&mut self.config.input.auto_commit_on_punctuation, "");
            ui.end_row();

            ui.separator();
            ui.end_row();

            ui.label("启用 AI 短语预测:");
            ui.checkbox(&mut self.config.input.enable_phrase_prediction, "");
            ui.end_row();

            if self.config.input.enable_phrase_prediction {
                ui.label("AI 触发最小长度:");
                ui.add(egui::Slider::new(&mut self.config.input.min_ai_input_length, 1..=5));
                ui.end_row();
            }
        });
    }

    fn show_ui_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("界面设置");
        ui.add_space(10.0);

        egui::Grid::new("ui_grid").num_columns(2).spacing([10.0, 8.0]).show(ui, |ui| {
            ui.label("显示候选词序号:");
            ui.checkbox(&mut self.config.ui.show_candidate_numbers, "");
            ui.end_row();

            ui.label("竖排候选词列表:");
            ui.checkbox(&mut self.config.ui.vertical_candidate_list, "");
            ui.end_row();

            ui.label("字体大小:");
            ui.add(egui::Slider::new(&mut self.config.ui.font_size, 8..=24));
            ui.end_row();

            ui.label("自定义字体:");
            ui.horizontal(|ui| {
                if self.config.ui.font_family.is_none() {
                    self.config.ui.font_family = Some(String::new());
                }
                if let Some(ref mut font) = self.config.ui.font_family {
                    ui.text_edit_singleline(font);
                    if font.is_empty() {
                        *font = "默认".to_string();
                    }
                }
            });
            ui.end_row();
        });
    }

    fn show_dictionary_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("词典设置");
        ui.add_space(10.0);

        egui::Grid::new("dict_grid").num_columns(2).spacing([10.0, 8.0]).show(ui, |ui| {
            ui.label("系统词典路径:");
            ui.text_edit_singleline(&mut self.config.dictionary.system_dictionary);
            ui.end_row();

            ui.label("用户词典路径:");
            ui.text_edit_singleline(&mut self.config.dictionary.user_dictionary);
            ui.end_row();

            ui.label("启用词频学习:");
            ui.checkbox(&mut self.config.dictionary.enable_learning, "");
            ui.end_row();

            if self.config.dictionary.enable_learning {
                ui.label("最大词条数:");
                ui.add(egui::Slider::new(&mut self.config.dictionary.max_user_dictionary_size, 1000..=500000));
                ui.end_row();

                ui.label("词频衰减系数:");
                ui.add(egui::Slider::new(&mut self.config.dictionary.frequency_decay, 0.90..=1.0));
                ui.end_row();
            }
        });

        ui.add_space(10.0);
        ui.collapsing("词典说明", |ui| {
            ui.label("系统词典: 只读，包含常用词汇");
            ui.label("用户词典: 可写，记录个人使用习惯");
            ui.label("词频衰减: 较少使用的词会逐渐降低优先级");
        });
    }

    fn show_about_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("AI Later 输入法");
            ui.add_space(10.0);
            ui.label("版本 0.1.0");
            ui.add_space(20.0);

            ui.label("基于 Rust 开发的 fcitx5 智能输入法");
            ui.label("支持 AI 模型预测和本地词典");
            ui.add_space(20.0);

            ui.hyperlink_to("项目主页", "git@github.com:crazykun/ailater-im.git");
            ui.label("© 2026 AI Later Project");
        });
    }
}

/// 配置结构（与主项目保持一致）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    model: ModelConfig,
    input: InputConfig,
    ui: UIConfig,
    dictionary: DictionaryConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model: ModelConfig::default(),
            input: InputConfig::default(),
            ui: UIConfig::default(),
            dictionary: DictionaryConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelConfig {
    model_type: String,
    api_endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    local_model_path: Option<String>,
    model_name: String,
    max_tokens: u32,
    temperature: f32,
    enable_cache: bool,
    cache_size: usize,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_type: "none".to_string(),
            api_endpoint: "http://localhost:8080/v1".to_string(),
            api_key: None,
            local_model_path: None,
            model_name: "qwen-0.8b".to_string(),
            max_tokens: 50,
            temperature: 0.7,
            enable_cache: true,
            cache_size: 10000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InputConfig {
    max_preedit_length: usize,
    num_candidates: usize,
    fuzzy_pinyin: bool,
    smart_correction: bool,
    page_size: usize,
    auto_commit_on_punctuation: bool,
    enable_phrase_prediction: bool,
    min_ai_input_length: usize,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            max_preedit_length: 64,
            num_candidates: 10,
            fuzzy_pinyin: true,
            smart_correction: true,
            page_size: 5,
            auto_commit_on_punctuation: true,
            enable_phrase_prediction: false,
            min_ai_input_length: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UIConfig {
    show_candidate_numbers: bool,
    vertical_candidate_list: bool,
    font_size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    font_family: Option<String>,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            show_candidate_numbers: true,
            vertical_candidate_list: false,
            font_size: 12,
            font_family: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DictionaryConfig {
    system_dictionary: String,
    user_dictionary: String,
    enable_learning: bool,
    max_user_dictionary_size: usize,
    frequency_decay: f32,
}

impl Default for DictionaryConfig {
    fn default() -> Self {
        Self {
            system_dictionary: "/usr/share/ailater-im/dict/system.dict".to_string(),
            user_dictionary: "~/.local/share/ailater-im/user.dict".to_string(),
            enable_learning: true,
            max_user_dictionary_size: 100000,
            frequency_decay: 0.99,
        }
    }
}
