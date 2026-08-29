#!/usr/bin/env bash
# inno-creed 초보자용 GUI 인스톨러 배포 zip을 만든다 (macOS / Linux).
#
# 전제: inno-creed 본체와 installer가 이미 release로 빌드돼 있어야 한다.
#   cargo build --release --bin inno-creed
#   cargo build --release -p installer
#
# macOS/Linux는 확장 프로그램 단계 자체가 없다(브라우저 쿠키를 직접 읽는
# 경로라서 native messaging host가 필요 없음 — docs/INSTALL.md 참고).
# 그래서 payload/에 extension/ 폴더를 넣지 않는다.
#
# exe 안에 exe를 내장하지 않는다 — installer와 payload/를 zip 안에서
# 나란히 두고, installer는 실행 시 자기 옆에서 payload를 찾는다.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OS="$(uname -s)"
case "$OS" in
  Darwin) TARGET_NAME="macos-arm64" ;;
  Linux)
    ARCH="$(uname -m)"
    case "$ARCH" in
      x86_64) TARGET_NAME="linux-x86_64" ;;
      aarch64) TARGET_NAME="linux-aarch64" ;;
      *) echo "지원하지 않는 아키텍처: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  *) echo "지원하지 않는 OS: $OS" >&2; exit 1 ;;
esac

for f in target/release/installer target/release/inno-creed; do
  if [ ! -f "$f" ]; then
    echo "$f 가 없습니다. 먼저 'cargo build --release --bin inno-creed' 와 'cargo build --release -p installer' 를 실행하세요." >&2
    exit 1
  fi
done

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

mkdir -p "$STAGE/payload"
cp target/release/installer "$STAGE/installer"
cp target/release/inno-creed "$STAGE/payload/inno-creed"
chmod +x "$STAGE/installer" "$STAGE/payload/inno-creed"

OUT_DIR="${1:-dist}"
mkdir -p "$OUT_DIR"
ZIP_PATH="$(cd "$OUT_DIR" && pwd)/inno-creed-installer-${TARGET_NAME}.zip"
rm -f "$ZIP_PATH"

(cd "$STAGE" && zip -r -q "$ZIP_PATH" .)

echo "만든 파일: $ZIP_PATH"
