//! inno-creed MCP 서버 (stdio).
//! 크레덴셜(Chrome 쿠키 복호화) 취득 → gw API 도구를 rmcp로 노출.

use anyhow::Result;
use inno_creed::{client::GwClient, creds, mcp::Amaranth, native_host};
use rmcp::{transport::stdio, ServiceExt};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // --version/-V: 인자 파싱기가 따로 없어 설치본 버전을 확인할 방법이 없었다.
    // 크레덴셜 취득(브라우저 쿠키 읽기) 전에 먼저 처리해 부작용 없이 즉시 종료한다.
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("inno-creed {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // --native-host: 브라우저 익스텐션이 Native Messaging으로 스폰하는 1회성 모드.
    // MCP 서버로 안 뜨고 메시지 하나 처리한 뒤 즉시 종료한다(자세한 이유는 native_host.rs).
    //
    // ⚠️ Chrome/Edge는 native host를 실행할 때 우리가 지정한 플래그를 안 붙이고, 대신
    // 확장앱 origin(`chrome-extension://<id>/`)을 인자로 넘긴다(Native Messaging 스펙
    // 그대로 — "path"가 곧 실행 커맨드라 우리 쪽 커스텀 인자를 끼워넣을 방법이 없다).
    // 그래서 `--native-host`뿐 아니라 origin 패턴도 같이 감지한다. `--native-host`는
    // 수동 테스트용으로 남겨둔다(브라우저는 안 쓰지만 CLI로 직접 찔러볼 때 편함).
    if args
        .iter()
        .any(|a| a == "--native-host" || a.starts_with("chrome-extension://") || a.starts_with("moz-extension://"))
    {
        return native_host::run();
    }

    // --install-extension-host [확장ID]: Chrome/Edge에 native messaging host를 등록하는
    // 1회성 설정 명령. 확장 ID 생략 시 이 저장소의 `extension/manifest.json`에 고정된
    // 기본값을 쓴다(직접 빌드한 익스텐션을 다른 키로 서명했다면 인자로 넘기면 됨).
    if args.iter().any(|a| a == "--install-extension-host") {
        let id = args
            .iter()
            .position(|a| a == "--install-extension-host")
            .and_then(|i| args.get(i + 1))
            .map(String::as_str)
            .unwrap_or(native_host::DEFAULT_EXTENSION_ID);
        native_host::install(id)?;
        return Ok(());
    }

    // 크리덴셜 취득 실패해도 서버는 뜬다(비치명적). 실패 시 도구 호출 시점에 로그인 안내를
    // tool 응답으로 반환한다(사용자가 채팅에서 볼 수 있게). 성공하면 캐시를 seed.
    let initial = match creds::from_browser() {
        Ok(c) => {
            eprintln!(
                "[inno-creed] 크레덴셜 취득 완료 (authToken {}자). MCP 서버 시작 (stdio)",
                c.auth_token.len()
            );
            Some(c)
        }
        Err(e) => {
            eprintln!(
                "[inno-creed] ⚠️ 크레덴셜 미취득 — 서버는 시작하되 도구 호출 시 로그인 안내를 반환합니다.\n{e}"
            );
            None
        }
    };

    // 세션 정보(compSeq/deptSeq/근태 empCd 등)는 첫 도구 호출 시 gw050A02로 lazy 취득 후
    // 10분 TTL 캐시된다(ensure_session). 시작 시 선취득하지 않는다.
    let client = GwClient::new(initial);

    let service = Amaranth::new(client).serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
