// inno-creed 초보자용 GUI 인스톨러 — inno-creed 본체와 완전히 분리된 별개 산출물.
// 배포 zip에 installer와 나란히 놓인 payload/를 찾아 설치하고, config-kit으로
// claude_desktop_config.json에 등록한다.

// 기본 "콘솔" 서브시스템으로 빌드되면 GUI 창과 별개로 검은 콘솔창이 뒤에 함께 뜬다
// (Windows 전용 속성 — 다른 OS는 원래 이런 구분이 없어 그냥 무시된다).
#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod install;
mod payload;
#[cfg(target_os = "windows")]
mod registry;

fn main() -> eframe::Result<()> {
    let uninstall = std::env::args().any(|a| a == "--uninstall");
    let title = if uninstall { "inno-creed 제거" } else { "inno-creed 설치" };
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
        .expect("bundled icon.png must decode");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 480.0])
            .with_resizable(false)
            .with_icon(icon),
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
            // Monospace에도 넣는다 — doctor 전체 출력을 `ui.monospace`로 그리는데
            // 여기에 없으면 그 안의 한글만 네모로 깨진다. 이쪽은 맨 뒤에 붙여
            // ASCII는 고정폭 그대로 두고 한글만 넘어오게 한다.
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("korean".to_owned());
        }
    }
    ctx.set_fonts(fonts);
}
