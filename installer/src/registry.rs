//! Windows "프로그램 추가/제거" 목록 등록 — 선택 기능. 없어도 inno-creed는 정상
//! 동작하지만, 있으면 초보자가 낯선 방식(zip에서 다시 installer 찾기)이 아니라
//! 익숙한 설정 화면에서 제거할 수 있다.
//!
//! `reg.exe`를 쓰는 이유는 `native_host.rs`와 같다 — Windows에 항상 있는 도구라
//! 레지스트리 FFI를 새로 안 만들어도 된다.

#![cfg(target_os = "windows")]

use std::os::windows::process::CommandExt;
use std::path::Path;

const UNINSTALL_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\inno-creed";
// GUI 앱에서 reg.exe를 부르면 순간적으로 검은 콘솔창이 깜빡인다 — 안 뜨게 한다.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn register_uninstall_entry(install_dir: &Path, installer_exe: &Path) -> anyhow::Result<()> {
    let uninstall_cmd = format!("\"{}\" --uninstall", installer_exe.display());
    add_string("DisplayName", "inno-creed")?;
    add_string("UninstallString", &uninstall_cmd)?;
    add_string("InstallLocation", &install_dir.display().to_string())?;
    add_string("Publisher", "inno-creed")?;
    add_string("DisplayIcon", &installer_exe.display().to_string())?;
    add_dword("NoModify", 1)?;
    add_dword("NoRepair", 1)?;
    Ok(())
}

pub fn remove_uninstall_entry() {
    let _ = std::process::Command::new("reg")
        .args(["delete", UNINSTALL_KEY, "/f"])
        .creation_flags(CREATE_NO_WINDOW)
        .status();
}

fn add_string(name: &str, value: &str) -> anyhow::Result<()> {
    run_reg_add(name, "REG_SZ", value)
}

fn add_dword(name: &str, value: u32) -> anyhow::Result<()> {
    run_reg_add(name, "REG_DWORD", &value.to_string())
}

fn run_reg_add(name: &str, ty: &str, value: &str) -> anyhow::Result<()> {
    let status = std::process::Command::new("reg")
        .args(["add", UNINSTALL_KEY, "/v", name, "/t", ty, "/d", value, "/f"])
        .creation_flags(CREATE_NO_WINDOW)
        .status()?;
    anyhow::ensure!(status.success(), "레지스트리 등록 실패: {name}");
    Ok(())
}
