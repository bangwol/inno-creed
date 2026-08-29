//! 실제 설치 동작 — payload 복사, config-kit으로 등록, (Windows만) 확장 브릿지 연결.
//!
//! 어디서 payload를 찾는지는 이 모듈이 몰라도 된다(그건 `payload` 모듈의 일) — 여기는
//! 이미 찾은 구체적인 경로만 받는다. 그래야 가짜 경로를 넣어 로직만 따로 테스트할 수 있다.

use std::path::{Path, PathBuf};

pub struct InstallResult {
    pub exe_path: PathBuf,
    pub extension_dir: Option<PathBuf>,
    pub backup_path: Option<PathBuf>,
}

pub fn perform_install(
    config_path: &Path,
    install_dir: &Path,
    src_bin: &Path,
    src_extension_dir: Option<&Path>,
    dest_bin_name: &str,
) -> anyhow::Result<InstallResult> {
    std::fs::create_dir_all(install_dir)?;

    let dest_bin = install_dir.join(dest_bin_name);
    std::fs::copy(src_bin, &dest_bin)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(&dest_bin)?.permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(&dest_bin, perm)?;
    }

    let backup_path = config_kit::backup(config_path)?;
    let mut root = config_kit::read_json(config_path)?;
    config_kit::merge_inno_creed_entry(&mut root, &dest_bin);
    config_kit::write_atomic(config_path, &root)?;

    let mut extension_dir = None;
    if let Some(src_ext) = src_extension_dir.filter(|p| p.exists()) {
        let dest_ext = install_dir.join("extension");
        copy_dir_all(src_ext, &dest_ext)?;
        // native_host.rs가 이미 구현한 등록 절차를 그대로 재사용 — installer가
        // 레지스트리/매니페스트 작성 로직을 다시 구현하지 않는다. (Windows 전용 동작)
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new(&dest_bin)
                .arg("--install-extension-host")
                .status();
        }
        extension_dir = Some(dest_ext);
    }

    Ok(InstallResult {
        exe_path: dest_bin,
        extension_dir,
        backup_path,
    })
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

/// 설치된 inno-creed의 `doctor`를 실행해 결과 전문을 돌려준다.
pub fn run_doctor(exe_path: &Path) -> anyhow::Result<String> {
    let out = std::process::Command::new(exe_path).arg("doctor").output()?;
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    if !out.stderr.is_empty() {
        text.push_str("\n--- stderr ---\n");
        text.push_str(&String::from_utf8_lossy(&out.stderr));
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("installer-test-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn install_merges_config_and_copies_binary_without_extension() {
        let work = temp_dir("basic");
        let src_bin = work.join("fake-inno-creed");
        std::fs::write(&src_bin, b"not a real binary, just bytes").unwrap();

        let config_path = work.join("claude_desktop_config.json");
        std::fs::write(
            &config_path,
            serde_json::to_string(&json!({ "preferences": { "epitaxyPrefs": { "x": 1 } } })).unwrap(),
        )
        .unwrap();

        let install_dir = work.join("installed");
        let result =
            perform_install(&config_path, &install_dir, &src_bin, None, "inno-creed").unwrap();

        assert!(result.exe_path.exists());
        assert!(result.extension_dir.is_none());
        assert!(result.backup_path.is_some());

        let written = config_kit::read_json(&config_path).unwrap();
        assert_eq!(
            written["mcpServers"]["inno-creed"]["command"],
            result.exe_path.to_string_lossy().as_ref()
        );
        // 기존에 있던 무관한 키는 손대지 않아야 한다.
        assert_eq!(written["preferences"]["epitaxyPrefs"]["x"], 1);

        std::fs::remove_dir_all(&work).ok();
    }

    #[test]
    fn install_copies_extension_dir_when_given() {
        let work = temp_dir("ext");
        let src_bin = work.join("fake-inno-creed");
        std::fs::write(&src_bin, b"fake").unwrap();

        let ext_src = work.join("ext-src");
        std::fs::create_dir_all(&ext_src).unwrap();
        std::fs::write(ext_src.join("manifest.json"), b"{}").unwrap();

        let config_path = work.join("claude_desktop_config.json");
        let install_dir = work.join("installed");

        let result = perform_install(
            &config_path,
            &install_dir,
            &src_bin,
            Some(&ext_src),
            "inno-creed",
        )
        .unwrap();

        let ext_dir = result.extension_dir.expect("확장 폴더가 복사됐어야 함");
        assert!(ext_dir.join("manifest.json").exists());

        std::fs::remove_dir_all(&work).ok();
    }

    #[test]
    fn install_on_missing_config_creates_fresh_one() {
        let work = temp_dir("fresh");
        let src_bin = work.join("fake-inno-creed");
        std::fs::write(&src_bin, b"fake").unwrap();

        let config_path = work.join("claude_desktop_config.json"); // 존재하지 않는 파일
        let install_dir = work.join("installed");
        let result =
            perform_install(&config_path, &install_dir, &src_bin, None, "inno-creed").unwrap();

        assert!(result.backup_path.is_none()); // 원본이 없었으니 백업도 없다
        let written = config_kit::read_json(&config_path).unwrap();
        assert!(written["mcpServers"]["inno-creed"]["command"].is_string());

        std::fs::remove_dir_all(&work).ok();
    }
}
