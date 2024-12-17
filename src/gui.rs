use crate::os;
use crate::re;
use eframe::{
    egui::{self, ecolor::HexColor},
    NativeOptions,
};

pub fn configure_fonts(ctx: &egui::Context) {
    use egui::{FontData, FontDefinitions, FontFamily};
    let mut fonts = egui::FontDefinitions::default();

    // 加载自定义字体
    fonts.font_data.insert(
        "my_chinese_font".to_owned(),
        FontData::from_static(include_bytes!("./fonts/msyh.ttc")), // 字体路径
    );

    // 设置自定义字体为主要字体
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "my_chinese_font".to_owned());
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "my_chinese_font".to_owned());

    ctx.set_fonts(fonts);
}

pub fn configure_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::default();
    // visuals.dark_mode = true; // 使用深色主题
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(0, 140, 140); // 背景色
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(128, 0, 30); // 悬停时的背景色
    visuals.override_text_color = Some(egui::Color32::from_rgb(234, 89, 40)); // 文本颜色

    ctx.set_visuals(visuals);
}

#[derive(Default, Debug)]
struct ParamCheck {
    status: i32,     // 状态，成功 or 失败
    message: String, // 提示信息
}

#[derive(Default)]
pub struct RepkgApp {
    pub target: String, // 指定目录
    pub saved: String,  // 保存目录
    pub as_title: bool, // 以名称创建文件夹
    pub all_combine: bool, // 所有文件合并到一个文件夹
    pub addition_suffix: Vec<String>, // 需要添加保存的后缀名称

    search_results: Vec<String>, // 搜索结果
    status_message: String,      // 状态信息
    message: Option<String>,
}


impl eframe::App for RepkgApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("壁纸转换");
            ui.separator();

            // 输入文件名与路径
            ui.horizontal(|ui| {
                ui.label("壁纸大目录：");
                ui.text_edit_singleline(&mut self.target);
                ui.add_space(10.0); // 可选：在两个输入框之间增加间距
                ui.label("提取结果存放目录：");
                ui.text_edit_singleline(&mut self.saved);
            });
            ui.separator();

            // 按钮触发搜索
            if ui.button("开始转换").clicked() {
                let check_param = check_search_param(&self.target, &self.saved);
                if check_param.status == 1 {
                    self.message = Some(check_param.message);
                } else {
                    self.status_message = "正在转换...".to_string();

                    let argumets = re::Param {
                        target: self.target.clone(),
                        saved: self.saved.clone(),
                    };

                    if let Err(e) = re::extract(argumets) {
                        self.status_message = if self.search_results.is_empty() {
                            "未找到匹配的文件！".to_string()
                        } else {
                            format!("找到 {} 个文件！", self.search_results.len())
                        };
                    }
                }
            }

            ui.separator();
            // 显示状态信息
            ui.label(&self.status_message);

            // 显示搜索结果
            ui.separator();
            ui.label("搜索结果：");

            if let Some(mes) = self.message.clone() {
                // 创建一个错误窗口，并设置位置和大小
                egui::Window::new("错误")
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0)) // 锚点设置为居中
                    .resizable(false)
                    .collapsible(false)
                    .show(ui.ctx(), |ui| {
                        ui.label(mes); // 错误信息
                        ui.separator();
                        if ui.button("关闭").clicked() {
                            self.message = None; // 清空错误信息
                        }
                    });
            }
        });
    }
}


fn check_search_param(target: &str, saved: &str) -> ParamCheck {
    if target.is_empty() {
        return ParamCheck {
            status: 1,
            message: String::from("壁纸大文件不能为空"),
        };
    }
    if saved.is_empty() {
        return ParamCheck {
            status: 1,
            message: String::from("保存地址不能为空"),
        };
    }
    ParamCheck {
        status: 2,
        message: String::from(""),
    }
}







