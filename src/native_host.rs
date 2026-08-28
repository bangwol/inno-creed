//! 브라우저 익스텐션(Chrome/Edge, `extension/`)이 Native Messaging으로 보내주는 쿠키를
//! 받아 로컬 캐시 파일에 저장하는 쪽. `creds::from_extension_cache`가 그 파일을 읽는다.
//!
//! **왜 존재하는가**: Windows Chrome/Edge의 app-bound(v20) 쿠키 암호화는 호출자가 브라우저
//! 자신인지 경로로 검증하므로, inno-creed(제3자 프로세스)가 쿠키 DB를 직접 복호화하는 건
//! 설계상 항상 막힌다(`creds.rs` 모듈 문서 참고). 반면 익스텐션은 브라우저가 공식으로 열어준
//! `chrome.cookies` API로 평문 값을 바로 받을 수 있다 — 이 값을 로컬 프로세스로 옮기는
//! 유일한 non-소켓 경로가 Native Messaging이다(익스텐션은 리스닝 소켓을 못 연다. 브라우저가
//! 이 실행파일을 스폰해서 stdio를 파이프해준다).
//!
//! **프로토콜**: 4바이트 네이티브 바이트오더 길이 + 그만큼의 UTF-8 JSON, 양방향 동일
//! (Chrome/Edge Native Messaging 스펙 그대로).
//!
//! **호출 방식**: 브라우저가 `sendNativeMessage` 한 번마다 이 프로세스를 새로 스폰하고,
//! 응답 메시지 하나를 받으면 종료시킨다 — 그래서 이 프로세스는 메시지 하나 처리하고 바로
//! 끝나는 1회성이다. MCP 서버 본체(장수 프로세스)와는 완전히 별개 실행이고, 캐시 파일
//! 하나로만 이어진다(소켓 없음).

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::io::{Read, Write};

use crate::creds::extension_cache_path;

const HOST_NAME: &str = "com.innogrid.inno_creed";

/// `extension/manifest.json`의 `"key"`(고정 공개키)로부터 결정되는 확장 프로그램 ID.
/// 이 값이 고정돼 있어야 익스텐션을 다시 로드해도(경로가 바뀌어도) native host의
/// `allowed_origins`가 계속 맞는다 — `"key"` 없이 unpacked로 로드하면 로드 경로에 따라
/// ID가 매번 달라져 이 등록이 깨진다.
pub const DEFAULT_EXTENSION_ID: &str = "hpabcmnjaahhdenpdmfjlmkfjljdldbf";

fn read_message(stdin: &mut impl Read) -> Result<Value> {
    let mut len_buf = [0u8; 4];
    stdin
        .read_exact(&mut len_buf)
        .context("stdin에서 길이 프리픽스 읽기 실패(브라우저가 아닌 다른 곳에서 실행한 건 아닌지)")?;
    let len = u32::from_ne_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    stdin.read_exact(&mut buf).context("stdin 본문 읽기 실패")?;
    serde_json::from_slice(&buf).context("네이티브 메시지 JSON 파싱 실패")
}

fn write_message(stdout: &mut impl Write, value: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    stdout.write_all(&(bytes.len() as u32).to_ne_bytes())?;
    stdout.write_all(&bytes)?;
    stdout.flush()?;
    Ok(())
}

/// 메시지 하나 처리하고 종료. stdout에는 프로토콜 메시지 외 어떤 것도 써서는 안 된다
/// (브라우저가 stdout 전체를 프로토콜로 해석함) — 진단은 전부 stderr로.
pub fn run() -> Result<()> {
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();

    let msg = read_message(&mut stdin)?;

    if msg.get("clear").and_then(Value::as_bool) == Some(true) {
        let result = clear_cache();
        return write_message(&mut stdout, &ack(result));
    }

    let auth_token = msg.get("authToken").and_then(Value::as_str);
    let sign_key = msg.get("signKey").and_then(Value::as_str);
    let result = match (auth_token, sign_key) {
        (Some(at), Some(hk)) if !at.is_empty() && !hk.is_empty() => write_cache(at, hk),
        _ => Err(anyhow::anyhow!("authToken/signKey 누락 또는 빈 값")),
    };
    write_message(&mut stdout, &ack(result))
}

fn ack(result: Result<()>) -> Value {
    match result {
        Ok(()) => json!({ "ok": true }),
        Err(e) => json!({ "ok": false, "error": e.to_string() }),
    }
}

fn write_cache(auth_token: &str, sign_key: &str) -> Result<()> {
    let path = extension_cache_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let captured_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let payload = json!({
        "authToken": auth_token,
        "signKey": sign_key,
        "capturedAtMs": captured_at_ms,
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&payload)?)
        .with_context(|| format!("캐시 파일 쓰기 실패: {}", path.display()))
}

fn clear_cache() -> Result<()> {
    let path = extension_cache_path()?;
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("캐시 파일 삭제 실패: {}", path.display())),
    }
}

/// Native Messaging 호스트를 등록한다 — 매니페스트 JSON을 쓰고 Chrome/Edge 레지스트리
/// 하이브 둘 다에 걸어준다(두 브라우저가 각자 다른 하이브를 본다). `reg.exe`를 쓰는 건
/// Windows에 항상 있는 도구라 레지스트리 FFI를 새로 안 만들어도 되기 때문 — 인자를
/// `Command::args`로 넘기므로(셸을 안 거침) 경로에 공백이 있어도 별도 이스케이프가
/// 필요 없다.
/// native host 매니페스트 경로. **`install`(쓰기)과 `doctor`(확인)가 공유한다** — 경로를 두
/// 군데 적으면 "등록했는데 doctor는 없다고 한다"가 생긴다. Windows 외에는 등록 자체가 없다.
#[cfg(target_os = "windows")]
pub fn manifest_path() -> Option<std::path::PathBuf> {
    let local = std::env::var("LOCALAPPDATA").ok()?;
    Some(std::path::PathBuf::from(format!("{local}\\inno-creed")).join(format!("{HOST_NAME}.json")))
}

#[cfg(not(target_os = "windows"))]
pub fn manifest_path() -> Option<std::path::PathBuf> {
    None
}

#[cfg(target_os = "windows")]
pub fn install(extension_id: &str) -> Result<()> {
    let exe = std::env::current_exe().context("실행파일 경로 취득 실패")?;
    let manifest_path = manifest_path().context("LOCALAPPDATA 없음")?;
    let manifest_dir = manifest_path.parent().context("매니페스트 부모 경로 없음")?;
    std::fs::create_dir_all(manifest_dir)?;

    let manifest = json!({
        "name": HOST_NAME,
        "description": "inno-creed 크레덴셜 브릿지 native messaging host",
        "path": exe.to_string_lossy(),
        "type": "stdio",
        "allowed_origins": [format!("chrome-extension://{extension_id}/")],
    });
    std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)
        .with_context(|| format!("매니페스트 쓰기 실패: {}", manifest_path.display()))?;
    eprintln!("[inno-creed] native host 매니페스트 작성: {}", manifest_path.display());

    for (browser, hive) in [
        ("Chrome", r"Software\Google\Chrome\NativeMessagingHosts"),
        ("Edge", r"Software\Microsoft\Edge\NativeMessagingHosts"),
    ] {
        let key = format!(r"HKCU\{hive}\{HOST_NAME}");
        let status = std::process::Command::new("reg")
            .args(["add", &key, "/ve", "/d", &manifest_path.to_string_lossy(), "/f"])
            .status()
            .with_context(|| format!("reg.exe 실행 실패({browser})"))?;
        if !status.success() {
            bail!("{browser} 레지스트리 등록 실패: {key}");
        }
        eprintln!("[inno-creed] {browser} native host 등록 완료: {key}");
    }
    eprintln!(
        "[inno-creed] 등록 완료. Chrome/Edge에서 확장 프로그램을 로드하면(chrome://extensions →\n\
         개발자 모드 → 압축해제된 확장 프로그램 로드 → extension/ 폴더) 로그인 즉시 크레덴셜이\n\
         자동으로 전달됩니다."
    );
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn install(_extension_id: &str) -> Result<()> {
    bail!("익스텐션 native host 등록은 현재 Windows만 지원합니다");
}
