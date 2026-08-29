// inno-creed 초보자용 GUI 인스톨러 — inno-creed 본체와 완전히 분리된 별개 산출물.
// 배포 zip에 installer와 나란히 놓인 payload/를 찾아 설치하고, config-kit으로
// claude_desktop_config.json에 등록한다.

mod app;
mod install;
mod payload;
#[cfg(target_os = "windows")]
mod registry;

fn main() -> eframe::Result<()> {
    let uninstall = std::env::args().any(|a| a == "--uninstall");
    let title = if uninstall { "inno-creed 제거" } else { "inno-creed 설치" };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 480.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        title,
        options,
        Box::new(move |cc| {
            setup_korean_font(&cc.egui_ctx);
            let app = if uninstall {
                app::InstallerApp::new_uninstall()
            } else {
                app::InstallerApp::default()
            };
            Ok(Box::new(app))
        }),
    )
}

/// egui 기본 폰트에는 한글 글리프가 없어 아무 설정 없이 그리면 네모(tofu)만 뜬다.
/// 각 OS에 기본으로 깔린 한글 폰트를 읽어 등록한다.
fn setup_korean_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let candidates = [
        "C:/Windows/Fonts/malgun.ttf",
        "/System/Library/Fonts/Supplemental/AppleGothic.ttf",
        "/usr/share/fonts/truetype/nanum/NanumGothic.ttf",
    ];
    if let Some(path) = candidates.iter().find(|p| std::path::Path::new(p).exists()) {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                "korean".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "korean".to_owned());
        }
    }
    ctx.set_fonts(fonts);
}
