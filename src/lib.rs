pub mod os;
pub mod gui;
pub mod re;
use std::env;

pub fn run() -> Result<(), eframe::Error>  {
    env::set_var("PNG_WARNINGS", "0");
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "壁纸提取",
        options,
        Box::new(|cc| {
            gui::configure_fonts(&cc.egui_ctx); // 配置字体
            gui::configure_theme(&cc.egui_ctx);
            Box::new(gui::RepkgApp::default())
        }),
    )
}