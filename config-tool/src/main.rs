#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use std::path::PathBuf;

// Re-export configuration types from the main library
use ailater_im::config::Config;

/// AI Later 配置工具
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 520.0])
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
    let mut fonts = egui::FontDefinitions::default();

    let font_paths = [
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/System/Library/Fonts/PingFang.ttc",
    ];

    for path in font_paths {
        if std::path::Path::new(path).exists() {
            if let Ok(font_data) = std::fs::read(path) {
                fonts
                    .font_data
                    .insert("cjk".to_owned(), egui::FontData::from_owned(font_data));
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
    config: Config,
    config_path: PathBuf,
    status_message: String,
    status_type: StatusType,
    selected_tab: ConfigTab,
    show_modified_warning: bool,
}

#[derive(Default, PartialEq, Eq)]
enum ConfigTab {
    #[default]
    Model,
    Input,
    UI,
    Dictionary,
    About,
}

#[derive(Default, PartialEq, Eq)]
enum StatusType {
    #[default]
    None,
    Success,
    Error,
    Info,
}

impl ConfigApp {
    fn default() -> Self {
        let config_path = directories::ProjectDirs::from("org.fcitx", "Fcitx5", "ailater-im")
            .and_then(|dirs| Some(dirs.config_dir().join("config.toml")))
            .unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".config/ailater-im/config.toml")
            });

        let config = if config_path.exists() {
            Self::load_config(&config_path).unwrap_or_default()
        } else {
            Config::default()
        };

        Self {
            config,
            config_path,
            status_message: "配置已加载".to_string(),
            status_type: StatusType::Info,
            selected_tab: ConfigTab::Model,
            show_modified_warning: false,
        }
    }

    fn load_config(path: &PathBuf) -> Option<Config> {
        let content = std::fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }

    fn save_config(&mut self) {
        if let Some(parent) = self.config_path.parent() {
            if std::fs::create_dir_all(parent).is_err() {
                self.show_status("创建配置目录失败", StatusType::Error);
                return;
            }
        }

        let toml_str = match toml::to_string_pretty(&self.config) {
            Ok(s) => s,
            Err(e) => {
                self.show_status(&format!("序列化配置失败: {}", e), StatusType::Error);
                return;
            }
        };

        match std::fs::write(&self.config_path, toml_str) {
            Ok(_) => {
                self.show_status("配置已保存", StatusType::Success);
                self.show_modified_warning = false;
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
        self.show_modified_warning = false;
    }

    fn mark_modified(&mut self) {
        if !self.show_modified_warning {
            self.show_modified_warning = true;
        }
    }
}

impl eframe::App for ConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("保存配置 (Ctrl+S)").clicked() {
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
            ui.horizontal(|ui| {
                let icon = match self.status_type {
                    StatusType::Success => "✓",
                    StatusType::Error => "✗",
                    StatusType::Info => "ℹ",
                    StatusType::None => "",
                };
                ui.label(format!("{} {}", icon, self.status_message));
                ui.separator();

                if self.show_modified_warning {
                    ui.colored_label(egui::Color32::YELLOW, "⚠ 配置已修改，请记得保存");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("配置文件: {}", self.config_path.display()));
                });
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
                        self.config = Config::default();
                        self.show_status("已重置为默认配置", StatusType::Info);
                        self.mark_modified();
                    }
                    if ui.button("📂 打开配置目录").clicked() {
                        if let Some(parent) = self.config_path.parent() {
                            if let Err(e) = open::that(parent) {
                                self.show_status(
                                    &format!("打开目录失败: {}", e),
                                    StatusType::Error,
                                );
                            }
                        }
                    }
                });
            });
        });

        // Keyboard shortcuts
        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::COMMAND, egui::Key::S) {
                self.save_config();
            }
        });
    }
}

impl ConfigApp {
    fn show_model_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("AI 模型设置");
        ui.add_space(10.0);

        egui::Grid::new("model_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("模型类型:");
                ui.horizontal(|ui| {
                    let mut changed = false;
                    if ui
                        .radio_value(
                            &mut self.config.model.model_type,
                            "none".to_string(),
                            "禁用 (仅本地词典)",
                        )
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .radio_value(
                            &mut self.config.model.model_type,
                            "remote".to_string(),
                            "远程 API",
                        )
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .radio_value(
                            &mut self.config.model.model_type,
                            "local".to_string(),
                            "本地模型",
                        )
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .radio_value(
                            &mut self.config.model.model_type,
                            "hybrid".to_string(),
                            "混合模式",
                        )
                        .changed()
                    {
                        changed = true;
                    }
                    if changed {
                        self.mark_modified();
                    }
                });
                ui.end_row();

                if self.config.model.model_type != "none" {
                    ui.label("API 端点:");
                    if ui
                        .text_edit_singleline(&mut self.config.model.api_endpoint)
                        .changed()
                    {
                        self.mark_modified();
                    }
                    ui.end_row();

                    ui.label("模型名称:");
                    if ui
                        .text_edit_singleline(&mut self.config.model.model_name)
                        .changed()
                    {
                        self.mark_modified();
                    }
                    ui.end_row();

                    ui.label("最大 Token 数:");
                    if ui
                        .add(egui::Slider::new(
                            &mut self.config.model.max_tokens,
                            10..=500,
                        ))
                        .changed()
                    {
                        self.mark_modified();
                    }
                    ui.end_row();

                    ui.label("温度 (0.0 - 2.0):");
                    if ui
                        .add(egui::Slider::new(
                            &mut self.config.model.temperature,
                            0.0..=2.0,
                        ))
                        .changed()
                    {
                        self.mark_modified();
                    }
                    ui.end_row();

                    ui.label("启用缓存:");
                    if ui
                        .checkbox(&mut self.config.model.enable_cache, "")
                        .changed()
                    {
                        self.mark_modified();
                    }
                    ui.end_row();

                    if self.config.model.enable_cache {
                        ui.label("缓存大小:");
                        if ui
                            .add(egui::Slider::new(
                                &mut self.config.model.cache_size,
                                100..=50000,
                            ))
                            .changed()
                        {
                            self.mark_modified();
                        }
                        ui.end_row();
                    }

                    ui.label("API Key:");
                    if ui
                        .text_edit_singleline(
                            self.config
                                .model
                                .api_key
                                .get_or_insert_with(Default::default),
                        )
                        .changed()
                    {
                        self.mark_modified();
                    }
                    ui.end_row();
                }

                if self.config.model.model_type == "local"
                    || self.config.model.model_type == "hybrid"
                {
                    ui.label("本地模型路径:");
                    if ui
                        .text_edit_singleline(
                            self.config
                                .model
                                .local_model_path
                                .get_or_insert_with(Default::default),
                        )
                        .changed()
                    {
                        self.mark_modified();
                    }
                    ui.end_row();
                }
            });

        ui.add_space(10.0);
        ui.collapsing("说明", |ui| {
            ui.label("• 禁用模式: 仅使用本地词典，不调用 AI");
            ui.label("• 远程 API: 调用 HTTP 接口的 AI 服务");
            ui.label("• 本地模型: 使用本地加载的 AI 模型");
            ui.label("• 混合模式: 优先远程，失败时回退到本地");
        });
    }

    fn show_input_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("输入设置");
        ui.add_space(10.0);

        egui::Grid::new("input_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("最大输入长度:");
                if ui
                    .add(egui::Slider::new(
                        &mut self.config.input.max_preedit_length,
                        10..=100,
                    ))
                    .changed()
                {
                    self.mark_modified();
                }
                ui.end_row();

                ui.label("候选词数量:");
                if ui
                    .add(egui::Slider::new(
                        &mut self.config.input.num_candidates,
                        5..=20,
                    ))
                    .changed()
                {
                    self.mark_modified();
                }
                ui.end_row();

                ui.label("每页候选词数:");
                if ui
                    .add(egui::Slider::new(&mut self.config.input.page_size, 3..=10))
                    .changed()
                {
                    self.mark_modified();
                }
                ui.end_row();

                ui.label("模糊拼音:");
                if ui
                    .checkbox(
                        &mut self.config.input.fuzzy_pinyin,
                        "支持 zh/z、ch/c、sh/s 等混淆",
                    )
                    .changed()
                {
                    self.mark_modified();
                }
                ui.end_row();

                ui.label("智能纠错:");
                if ui
                    .checkbox(&mut self.config.input.smart_correction, "")
                    .changed()
                {
                    self.mark_modified();
                }
                ui.end_row();

                ui.label("标点自动上屏:");
                if ui
                    .checkbox(&mut self.config.input.auto_commit_on_punctuation, "")
                    .changed()
                {
                    self.mark_modified();
                }
                ui.end_row();

                ui.separator();
                ui.end_row();

                ui.label("启用 AI 短语预测:");
                if ui
                    .checkbox(&mut self.config.input.enable_phrase_prediction, "")
                    .changed()
                {
                    self.mark_modified();
                }
                ui.end_row();

                if self.config.input.enable_phrase_prediction {
                    ui.label("AI 触发最小长度:");
                    if ui
                        .add(egui::Slider::new(
                            &mut self.config.input.min_ai_input_length,
                            1..=5,
                        ))
                        .changed()
                    {
                        self.mark_modified();
                    }
                    ui.end_row();
                }
            });
    }

    fn show_ui_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("界面设置");
        ui.add_space(10.0);

        egui::Grid::new("ui_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("显示候选词序号:");
                if ui
                    .checkbox(&mut self.config.ui.show_candidate_numbers, "")
                    .changed()
                {
                    self.mark_modified();
                }
                ui.end_row();

                ui.label("竖排候选词列表:");
                if ui
                    .checkbox(&mut self.config.ui.vertical_candidate_list, "")
                    .changed()
                {
                    self.mark_modified();
                }
                ui.end_row();

                ui.label("字体大小:");
                if ui
                    .add(egui::Slider::new(&mut self.config.ui.font_size, 8..=24))
                    .changed()
                {
                    self.mark_modified();
                }
                ui.end_row();

                ui.label("自定义字体:");
                ui.horizontal(|ui| {
                    if self.config.ui.font_family.is_none() {
                        self.config.ui.font_family = Some(String::new());
                    }
                    let changed = {
                        let font = self.config.ui.font_family.as_mut().unwrap();
                        ui.text_edit_singleline(font).changed()
                    };
                    if changed {
                        self.mark_modified();
                    }
                    let font = self.config.ui.font_family.as_mut().unwrap();
                    if font.is_empty() {
                        *font = "默认".to_string();
                    }
                });
                ui.end_row();
            });
    }

    fn show_dictionary_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("词典设置");
        ui.add_space(10.0);

        egui::Grid::new("dict_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("系统词典路径:");
                if ui
                    .text_edit_singleline(&mut self.config.dictionary.system_dictionary)
                    .changed()
                {
                    self.mark_modified();
                }
                ui.end_row();

                ui.label("用户词典路径:");
                if ui
                    .text_edit_singleline(&mut self.config.dictionary.user_dictionary)
                    .changed()
                {
                    self.mark_modified();
                }
                ui.end_row();

                ui.label("启用词频学习:");
                if ui
                    .checkbox(&mut self.config.dictionary.enable_learning, "")
                    .changed()
                {
                    self.mark_modified();
                }
                ui.end_row();

                if self.config.dictionary.enable_learning {
                    ui.label("最大词条数:");
                    if ui
                        .add(egui::Slider::new(
                            &mut self.config.dictionary.max_user_dictionary_size,
                            1000..=500000,
                        ))
                        .changed()
                    {
                        self.mark_modified();
                    }
                    ui.end_row();

                    ui.label("词频衰减系数:");
                    if ui
                        .add(egui::Slider::new(
                            &mut self.config.dictionary.frequency_decay,
                            0.90..=1.0,
                        ))
                        .changed()
                    {
                        self.mark_modified();
                    }
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
            ui.label(format!("版本 {}", env!("CARGO_PKG_VERSION")));
            ui.add_space(20.0);

            ui.label("基于 Rust 开发的 fcitx5 智能输入法");
            ui.label("支持 AI 模型预测和本地词典");
            ui.add_space(20.0);

            ui.hyperlink_to("项目主页", "https://github.com/crazykun/ailater-im#");
            ui.label("© 2026 AI Later Project");

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            ui.label("配置结构由主库 (ailater-im) 提供");
            ui.label("通过 Cargo Workspace 统一管理");
        });
    }
}
