//! 마법사 상태 머신 — 화면 전환과 각 화면의 렌더링.

use crate::{install, payload};
#[cfg(target_os = "windows")]
use crate::registry;
use eframe::egui;
use std::path::PathBuf;

enum Screen {
    Welcome,
    Confirm,
    AppRunning,
    Installing,
    ExtensionGuide,
    Done,
    Error(String),
    UninstallConfirm,
    UninstallAppRunning,
    UninstallDone,
}

pub struct InstallerApp {
    screen: Screen,
    config_path: Option<PathBuf>,
    config_candidates: Vec<PathBuf>,
    install_dir: PathBuf,
    install_result: Option<install::InstallResult>,
    doctor_output: Option<String>,
    doctor_ok: bool,
    doctor_expanded: bool,
}

impl Default for InstallerApp {
    fn default() -> Self {
        let candidates = config_kit::desktop_config_candidates();
        let found = candidates.iter().find(|p| p.exists()).cloned();
        Self {
            screen: Screen::Welcome,
            config_path: found,
            config_candidates: candidates,
            install_dir: payload::default_install_dir(),
            install_result: None,
            doctor_output: None,
            doctor_ok: false,
            doctor_expanded: false,
        }
    }
}

impl InstallerApp {
    /// `--uninstall`로 실행됐을 때의 초기 상태. 설치 때와 같은 자동 감지 로직으로
    /// config 경로·설치 위치를 잡는다 — 별도 메타데이터 파일을 두지 않는다.
    pub fn new_uninstall() -> Self {
        Self {
            screen: Screen::UninstallConfirm,
            ..Self::default()
        }
    }
}

impl eframe::App for InstallerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(20.0);
            match &self.screen {
                Screen::Welcome => self.welcome_screen(ui),
                Screen::Confirm => self.confirm_screen(ui),
                Screen::AppRunning => self.app_running_screen(ui),
                Screen::Installing => self.installing_screen(ui),
                Screen::ExtensionGuide => self.extension_guide_screen(ui),
                Screen::Done => self.done_screen(ui),
                Screen::Error(_) => self.error_screen(ui),
                Screen::UninstallConfirm => self.uninstall_confirm_screen(ui),
                Screen::UninstallAppRunning => self.uninstall_app_running_screen(ui),
                Screen::UninstallDone => self.uninstall_done_screen(ui),
            }
        });
    }
}

impl InstallerApp {
    fn welcome_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("inno-creed 설치");
            ui.add_space(12.0);
            ui.label("이노그리드 아마란스를 Claude로 다루는 inno-creed를 설치합니다.");
            ui.label("Claude Desktop 앱의 채팅·Cowork·Code 탭에서 곧바로 쓸 수 있게 등록해 드립니다.");
            ui.add_space(20.0);

            ui.group(|ui| {
                ui.set_width(460.0);
                ui.label(
                    "⚠️  아직 Claude Desktop이 없다면, claude.ai/download에서 먼저 설치한 뒤 \
                     이 프로그램을 다시 실행하세요.",
                );
            });
            ui.add_space(12.0);
            ui.group(|ui| {
                ui.set_width(460.0);
                ui.label(
                    "이 설치 프로그램은 아직 코드 서명이 되어 있지 않습니다. Windows가 \
                     \"PC를 보호했습니다\" 경고를 띄우면 [추가 정보] → [실행]을 눌러주세요.",
                );
            });

            ui.add_space(28.0);
            if ui
                .add(egui::Button::new("다음 →").min_size(egui::vec2(140.0, 36.0)))
                .clicked()
            {
                if let Err(e) = payload::verify_payload_present() {
                    self.screen = Screen::Error(e);
                } else {
                    self.screen = Screen::Confirm;
                }
            }
        });
    }

    fn confirm_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("설치 위치 확인");
        ui.add_space(16.0);

        ui.label("Claude Desktop 설정 파일:");
        match &self.config_path {
            Some(p) => {
                ui.monospace(p.display().to_string());
            }
            None => {
                ui.colored_label(
                    egui::Color32::from_rgb(200, 90, 60),
                    "찾지 못했습니다. Claude Desktop을 한 번이라도 실행한 적이 있나요?",
                );
                for c in &self.config_candidates {
                    ui.small(format!("  (확인한 경로) {}", c.display()));
                }
            }
        }
        if ui.button("직접 선택...").clicked() {
            if let Some(picked) = rfd::FileDialog::new()
                .add_filter("Claude 설정 파일", &["json"])
                .set_file_name("claude_desktop_config.json")
                .pick_file()
            {
                self.config_path = Some(picked);
            }
        }

        ui.add_space(16.0);
        ui.label("inno-creed를 놓을 위치:");
        ui.monospace(self.install_dir.display().to_string());
        if ui.button("다른 폴더 선택...").clicked() {
            if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                self.install_dir = dir.join("inno-creed");
            }
        }

        ui.add_space(24.0);
        ui.horizontal(|ui| {
            if ui.button("← 이전").clicked() {
                self.screen = Screen::Welcome;
            }
            let can_install = self.config_path.is_some();
            if ui
                .add_enabled(can_install, egui::Button::new("설치").min_size(egui::vec2(120.0, 32.0)))
                .clicked()
            {
                self.screen = if config_kit::is_claude_desktop_running() {
                    Screen::AppRunning
                } else {
                    Screen::Installing
                };
            }
        });
    }

    fn app_running_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Claude Desktop을 종료해주세요");
            ui.add_space(12.0);
            ui.label("설정 파일을 안전하게 쓰려면 Claude Desktop이 완전히 꺼져 있어야 합니다.");
            ui.label("(창을 닫아도 트레이에 남아있을 수 있습니다 — 트레이 아이콘에서도 종료해주세요.)");
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                if ui.button("다시 확인").clicked() {
                    self.screen = if config_kit::is_claude_desktop_running() {
                        Screen::AppRunning
                    } else {
                        Screen::Installing
                    };
                }
                if ui.button("← 이전").clicked() {
                    self.screen = Screen::Confirm;
                }
            });
        });
    }

    fn installing_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("설치 중...");
        });
        let Some(config_path) = self.config_path.clone() else {
            self.screen = Screen::Error("설정 파일 경로가 지정되지 않았습니다.".into());
            return;
        };
        let src_bin = payload::payload_binary_path();
        let src_ext = payload::payload_extension_dir();
        let src_ext = src_ext.exists().then_some(src_ext.as_path());
        match install::perform_install(
            &config_path,
            &self.install_dir,
            &src_bin,
            src_ext,
            payload::inno_creed_binary_name(),
        ) {
            Ok(result) => {
                // "프로그램 추가/제거" 등록은 있으면 좋은 부가 기능이라, 실패해도
                // 설치 자체를 막지 않는다(레지스트리 접근이 막힌 사내 정책 등 대비).
                #[cfg(target_os = "windows")]
                {
                    if let Ok(installer_copy) =
                        install::copy_installer_self(&self.install_dir, "installer.exe")
                    {
                        let _ = registry::register_uninstall_entry(&self.install_dir, &installer_copy);
                    }
                }
                self.screen = if result.extension_dir.is_some() {
                    Screen::ExtensionGuide
                } else {
                    Screen::Done
                };
                self.install_result = Some(result);
            }
            Err(e) => self.screen = Screen::Error(format!("설치 중 오류가 발생했습니다: {e:#}")),
        }
    }

    fn extension_guide_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("확장 프로그램 연결");
        ui.add_space(12.0);
        ui.label("아마란스 로그인 정보를 안전하게 가져오려면 Chrome/Edge 확장 프로그램을 마저 등록해야 합니다.");
        ui.add_space(8.0);
        ui.label("1. 아래 [확장 폴더 열기]로 열리는 폴더를 기억해두세요.");
        ui.label("2. chrome://extensions (또는 edge://extensions)를 열고 우측 상단 개발자 모드를 켭니다.");
        ui.label("3. [압축해제된 확장 프로그램을 로드합니다]를 눌러 방금 그 폴더를 선택합니다.");
        ui.add_space(16.0);

        ui.horizontal(|ui| {
            if let Some(ext_dir) = self.install_result.as_ref().and_then(|r| r.extension_dir.clone()) {
                if ui.button("📁 확장 폴더 열기").clicked() {
                    let _ = open::that(ext_dir);
                }
            }
            if ui.button("🌐 chrome://extensions 열기").clicked() {
                let _ = open::that("chrome://extensions");
            }
        });

        ui.add_space(24.0);
        if ui
            .add(egui::Button::new("다음 →").min_size(egui::vec2(140.0, 36.0)))
            .clicked()
        {
            self.screen = Screen::Done;
        }
    }

    fn done_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("설치 완료");
            ui.add_space(12.0);

            if self.doctor_output.is_none() {
                if let Some(result) = &self.install_result {
                    match install::run_doctor(&result.exe_path) {
                        Ok(out) => {
                            self.doctor_ok = out.contains("✅ 인증 성공");
                            self.doctor_output = Some(out);
                        }
                        Err(e) => {
                            self.doctor_output = Some(format!("doctor 실행 실패: {e:#}"));
                        }
                    }
                }
            }

            if self.doctor_ok {
                ui.colored_label(egui::Color32::from_rgb(40, 140, 60), "✅ 인증까지 확인됐습니다. 바로 쓸 수 있습니다.");
            } else {
                ui.colored_label(
                    egui::Color32::from_rgb(200, 140, 30),
                    "⚠️ 등록은 됐지만 인증 확인은 안 됐습니다 — 자세히 보기에서 원인을 확인하세요.",
                );
            }
            ui.label("Claude Desktop을 (다시) 실행하면 채팅·Cowork·Code 탭에서 inno-creed 도구를 쓸 수 있습니다.");

            if let Some(bak) = self.install_result.as_ref().and_then(|r| r.backup_path.clone()) {
                ui.add_space(6.0);
                ui.small(format!("기존 설정은 백업해뒀습니다: {}", bak.display()));
            }

            ui.add_space(12.0);
            ui.checkbox(&mut self.doctor_expanded, "자세히 보기 (doctor 전체 출력)");
            if self.doctor_expanded {
                if let Some(out) = &self.doctor_output {
                    egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                        ui.monospace(out);
                    });
                }
            }
        });
    }

    fn error_screen(&mut self, ui: &mut egui::Ui) {
        let message = if let Screen::Error(m) = &self.screen {
            m.clone()
        } else {
            String::new()
        };
        ui.vertical_centered(|ui| {
            ui.heading("문제가 발생했습니다");
            ui.add_space(12.0);
            ui.colored_label(egui::Color32::from_rgb(200, 60, 50), &message);
            ui.add_space(20.0);
            if ui.button("← 처음으로").clicked() {
                self.screen = Screen::Welcome;
            }
        });
    }

    fn uninstall_confirm_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("inno-creed 제거");
            ui.add_space(12.0);
            ui.label("Claude Desktop 설정에서 inno-creed 등록을 지우고, 설치된 파일을 삭제합니다.");
            ui.add_space(8.0);
            match &self.config_path {
                Some(p) => {
                    ui.small(format!("설정 파일: {}", p.display()));
                }
                None => {
                    ui.colored_label(egui::Color32::from_rgb(200, 90, 60), "설정 파일을 찾지 못했습니다 — 등록 해제는 건너뜁니다.");
                }
            }
            ui.small(format!("삭제할 폴더: {}", self.install_dir.display()));
            ui.add_space(20.0);
            if ui
                .add(egui::Button::new("제거").min_size(egui::vec2(120.0, 32.0)))
                .clicked()
            {
                self.screen = if config_kit::is_claude_desktop_running() {
                    Screen::UninstallAppRunning
                } else {
                    self.do_uninstall();
                    Screen::UninstallDone
                };
            }
        });
    }

    fn uninstall_app_running_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Claude Desktop을 종료해주세요");
            ui.add_space(12.0);
            ui.label("설정 파일을 안전하게 고치려면 Claude Desktop이 완전히 꺼져 있어야 합니다.");
            ui.add_space(20.0);
            if ui.button("다시 확인").clicked() {
                self.screen = if config_kit::is_claude_desktop_running() {
                    Screen::UninstallAppRunning
                } else {
                    self.do_uninstall();
                    Screen::UninstallDone
                };
            }
        });
    }

    fn do_uninstall(&mut self) {
        if let Some(config_path) = &self.config_path {
            let installer_copy = self.install_dir.join("installer.exe");
            if let Err(e) = install::perform_uninstall(config_path, &self.install_dir, &installer_copy) {
                self.screen = Screen::Error(format!("제거 중 오류가 발생했습니다: {e:#}"));
                return;
            }
            #[cfg(target_os = "windows")]
            {
                registry::remove_uninstall_entry();
                install::schedule_self_delete(&installer_copy, &self.install_dir);
            }
            #[cfg(not(target_os = "windows"))]
            {
                install::schedule_self_delete(&installer_copy, &self.install_dir);
            }
        }
    }

    fn uninstall_done_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("제거 완료");
            ui.add_space(12.0);
            ui.label("inno-creed 등록을 지웠습니다. Claude Desktop을 다시 실행하면 반영됩니다.");
            ui.add_space(8.0);
            ui.small("이 창은 닫아도 됩니다.");
        });
    }
}
