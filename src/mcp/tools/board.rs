//! 게시판 도구.
//!
//! 라우터는 `board_router`로 생성돼 `super::Amaranth::all_tools()`에서 합성된다.
//! 담당 도메인 로직은 `modules::board`에 있고, 여기 핸들러는 **`ensure_session` → 모듈 호출 → 감싸기**만 한다.

use rmcp::{handler::server::wrapper::Parameters, model::{CallToolResult, ContentBlock}, tool, tool_router, ErrorData};

use crate::mcp::{map_domain_err, Amaranth};
use crate::mcp::args::board::*;
use crate::modules;

#[tool_router(router = board_router, vis = "pub(crate)")]
impl Amaranth {
    #[tool(
        description = "게시판 최근 공지/게시글 목록을 조회한다(본문 프리뷰 포함). 검색어(field로 제목/내용/작성자 지정)·등록일 범위로 필터 가능. 응답: `{totalCnt, articles[]}` — 첨부 유무는 `fileCnt`(숫자, 0이면 없음), 첨부 조회용 키는 `attachmentUid`."
    )]
    async fn list_notices(
        &self,
        Parameters(a): Parameters<ListNoticesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let data = modules::board::list_notices(
            &self.client,
            a.page,
            a.page_size,
            &a.search,
            &a.field,
            &a.start_date,
            &a.end_date,
        )
        .await
        .map_err(map_domain_err)?;
        Ok(CallToolResult::success(vec![ContentBlock::text(data.to_string())]))
    }

    #[tool(
        description = "게시글 1건의 본문(평문)·댓글을 조회한다. 본문 삽입 이미지는 평문에 `[이미지]`로 자리가 남고 경로는 `images[]`로 나온다(순서 일치) — `download_body_image`로 받아볼 수 있다. **이미지는 정식 첨부가 아니라 `fileCnt`에 안 잡히므로**, fileCnt=0이어도 본문에 이미지가 있을 수 있다. ⚠️ 호출 시 조회수 증가(실제 열람 처리)."
    )]
    async fn read_notice(
        &self,
        Parameters(a): Parameters<ReadNoticeArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let data = modules::board::read_post(&self.client, &a.art_seq_no)
            .await
            .map_err(map_domain_err)?;
        Ok(CallToolResult::success(vec![ContentBlock::text(data.to_string())]))
    }

    #[tool(description = "게시글 첨부파일 목록을 조회한다. 응답 `{files[]}`의 각 항목에 **다운로드에 그대로 쓸 `fileSn`(0-base 인덱스)** 이 들어 있다. art_seq_no+uid(=목록의 attachmentUid) 필요.")]
    async fn list_notice_attachments(
        &self,
        Parameters(a): Parameters<ListAttachmentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let data = modules::board::list_attachments(&self.client, &a.art_seq_no, &a.uid)
            .await
            .map_err(map_domain_err)?;
        Ok(CallToolResult::success(vec![ContentBlock::text(data.to_string())]))
    }

    #[tool(
        description = "**본문에 삽입된 이미지**를 다운로드해 out_path에 저장한다 — 게시판·메일 공용. src에는 read_notice의 `images[]` 또는 read_mail의 `inlineImages[]` 값을 그대로 준다. 정식 첨부(`fileCnt`)와는 별개 경로라 download_*_attachment 로는 받을 수 없다. ⚠️ 외부 호스트 이미지(메일 서명 로고·추적 픽셀 등)는 거부한다 — 그래서 read_mail 은 그런 것을 `inlineImages`에 싣지 않고 `remoteResourceCount`로 개수만 알린다."
    )]
    async fn download_body_image(
        &self,
        Parameters(a): Parameters<DownloadBodyImageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let data = modules::board::download_body_image(&self.client, &a.src, &a.out_path)
            .await
            .map_err(map_domain_err)?;
        Ok(CallToolResult::success(vec![ContentBlock::text(data.to_string())]))
    }

    #[tool(description = "게시글 첨부파일을 다운로드해 out_path에 저장한다. **file_sn 은 list_notice_attachments 결과 `files[].fileSn`(0-base 인덱스)** — ⚠️ 메일 쪽 download_mail_attachment 의 file_sn(서버 토큰 문자열)과는 의미가 다르다.")]
    async fn download_notice_attachment(
        &self,
        Parameters(a): Parameters<DownloadAttachmentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let data =
            modules::board::download_attachment(&self.client, &a.art_seq_no, &a.uid, a.file_sn, &a.out_path)
                .await
                .map_err(map_domain_err)?;
        Ok(CallToolResult::success(vec![ContentBlock::text(data.to_string())]))
    }
}
