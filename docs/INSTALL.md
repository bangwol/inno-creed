# 설치 가이드

`inno-creed`는 아마란스(`gw.innogrid.com`) 그룹웨어를 다루는 **MCP 서버**입니다. 단독으로 실행하는 앱이 아니라, **Claude Code 같은 MCP 클라이언트에 등록해서** 대화로 사용합니다.

---

## 0. 전제 조건

- **MCP 클라이언트** — [Claude Code](https://claude.com/claude-code)(권장) 또는 stdio MCP를 지원하는 클라이언트. 이게 없으면 바이너리를 실행해도 아무 일도 안 합니다(입력을 기다리다 종료).
- **로그인된 브라우저** — Chrome/Edge 또는 Firefox로 `https://gw.innogrid.com` 에 로그인된 **데스크톱** 환경. (헤드리스 서버·외부인 사용 불가)
- **이노그리드 사내 계정.**
- **(Windows 권장) Chrome/Edge 확장 프로그램** — 4번 참고. 안 쓰면 쿠키 DB를 직접 읽는데, Windows에서는 세션쿠키·파일잠금·`v20` 암호화 때문에 구조적으로 잘 안 됩니다.

지원 바이너리: **macOS(Apple Silicon)**, **Linux x86_64 / aarch64**, **Windows x86_64**.
> Intel 맥용 바이너리는 제공하지 않습니다(필요하면 [소스 빌드](#부록-소스-빌드)).

> **(Linux) `libsecret-tools`가 필요합니다.** Chrome 쿠키를 GNOME Keyring/KWallet에서
> 자동 복호화하려면 `secret-tool`이 있어야 합니다. 없으면 미리 설치하세요:
> `sudo apt install libsecret-tools`(Debian/Ubuntu 계열, 데스크톱 세션에서 키링이
> 잠금 해제돼 있어야 합니다). 자세한 내용은 [6. 크레덴셜이 안 잡힐 때](#6-크레덴셜이-안-잡힐-때-문제-해결) 참고.

---

## 1. 다운로드

[**릴리즈 최신본**](https://github.com/zilhak/inno-creed/releases/latest)에서 OS에 맞는 파일을 받습니다.

| OS / arch | 파일 |
|---|---|
| macOS (Apple Silicon) | `inno-creed-macos-arm64` |
| Linux x86_64 | `inno-creed-linux-x86_64` |
| Linux aarch64 | `inno-creed-linux-aarch64` |
| Windows x86_64 | `inno-creed-windows-x86_64.exe` |

---

## 2. OS별 설치

### macOS (Apple Silicon)

```sh
cd ~/Downloads
chmod +x inno-creed-macos-arm64
xattr -d com.apple.quarantine inno-creed-macos-arm64    # Gatekeeper 차단 해제(미서명 바이너리)
mkdir -p ~/bin && mv inno-creed-macos-arm64 ~/bin/inno-creed
```
> Gatekeeper 경고가 뜨면 위 `xattr` 명령으로 해제하거나, Finder에서 **우클릭 → 열기**를 한 번 해줍니다.

### Linux (x86_64 / aarch64)

```sh
cd ~/Downloads
chmod +x inno-creed-linux-*
mkdir -p ~/bin && mv inno-creed-linux-* ~/bin/inno-creed
```

### Windows (x86_64)

1. `inno-creed-windows-x86_64.exe`를 원하는 폴더로 옮깁니다(예: `C:\Tools\inno-creed.exe`).
2. 처음 실행 시 SmartScreen **"Windows가 PC를 보호했습니다"** 창이 뜨면 → **추가 정보 → 실행**.

---

## 3. MCP 클라이언트에 등록

**Claude Code:**

```sh
claude mcp add inno-creed --scope user -- /절대경로/inno-creed          # Windows: ...\inno-creed.exe
```

> ⚠️ **`--scope user`를 빼먹으면 등록한 그 디렉토리에서만 보입니다**(기본값이 `local`). 다른 프로젝트에서 목록에 없으면 이걸 의심하세요 — `claude mcp list`로 확인합니다.

또는 설정 JSON에 직접:

```json
{
  "mcpServers": {
    "inno-creed": {
      "command": "/절대경로/inno-creed"
    }
  }
}
```

등록 후 **클라이언트를 재시작**하면 도구가 노출됩니다.

---

## 4. 크레덴셜 연결 — Chrome/Edge 확장 프로그램 (Windows 권장)

`gw.innogrid.com`의 로그인 쿠키(`BIZCUBE_AT`/`BIZCUBE_HK`)는 **세션 쿠키**라 브라우저가 켜져 있는 동안만 존재합니다. Windows Chrome/Edge는 여기에 더해 실행 중 쿠키 파일을 배타 잠금 걸고, `v20` app-bound 암호화도 제3자 프로세스로는 설계상 항상 거부합니다 — 쿠키 DB 파일을 직접 읽는 방식(5번 문제 해결 참고)은 이 조합을 다 뚫어야 하는 데다 [DBSC](#dbsc란-쿠키-db-직접-읽기가-왜-점점-막히나) 때문에 갈수록 막힙니다. **Windows에서는 Chrome/Edge 확장 프로그램을 쓰세요** — 브라우저가 공식으로 열어준 `cookies` API로 평문 값을 바로 받아 이 문제들을 전부 우회합니다.

1. Native messaging host를 등록합니다(최초 1회):
   ```sh
   inno-creed --install-extension-host
   ```
2. `chrome://extensions`(또는 `edge://extensions`)를 엽니다.
3. 우측 상단 **개발자 모드**를 켭니다.
4. **압축해제된 확장 프로그램을 로드합니다** → 이 저장소를 받은 경로의 `extension/` 폴더를 선택합니다.
5. `https://gw.innogrid.com`에 로그인돼 있으면(또는 방금 로그인하면) 자동으로 크레덴셜이 전달됩니다. 이후로도 로그인·로그아웃할 때마다 자동으로 동기화됩니다 — 매번 다시 로드할 필요 없습니다.

> Chrome/Edge 둘 다에서 쓰려면 익스텐션을 두 브라우저 각각에 로드하면 됩니다(같은 `extension/` 폴더, native host 등록은 이미 양쪽 다 돼 있음).

## 5. 로그인 & 확인 (macOS/Linux, 또는 Windows에서 확장 프로그램 없이)

1. **macOS/Linux**: Chrome 또는 Firefox로 `https://gw.innogrid.com` 에 로그인해 둡니다. **Windows**(확장 프로그램 없이 시도하는 경우): Chrome 또는 Edge로 로그인해 두되, `v20` app-bound 때문에 거의 항상 실패합니다(Windows Firefox는 애초에 지원하지 않음 — 아래 DBSC 섹션) — 4번 확장 프로그램을 쓰는 게 사실상 유일한 방법입니다.
2. (macOS + Chrome) 첫 실행 시 키체인 `Chrome Safe Storage` 접근 허용 프롬프트가 **1회** 뜹니다 → 허용.
3. 정상 기동이면 로그에 이렇게 찍힙니다:
   ```
   [inno-creed] 크레덴셜 취득 완료 (authToken NN자). MCP 서버 시작 (stdio)
   ```
   MCP 클라이언트에서 `list_resources` 같은 도구가 보이면 성공입니다.

---

## DBSC란 — 쿠키 DB 직접 읽기가 왜 점점 막히나

Chrome은 **Device Bound Session Credentials(DBSC)**를 2026년 4월(Chrome 146) Windows GA로 켜서, 세션 쿠키를 기기에 암호학적으로 묶어 브라우저 프로세스 밖에서 파일·COM으로 훔쳐 쓰지 못하게 막습니다(관리자 설정으로도 못 끔). Edge도 같은 Chromium이라 뒤따를 걸로 보입니다(2025년 10월 Origin Trial 종료, GA 미발표). 쿠키 DB 직접 읽기는 **지금은 운 좋게 되더라도 가까운 미래에 완전히 막힐 걸 전제로** 쓰세요. 확장 프로그램 경로(4번)는 브라우저 자신의 공식 `cookies` API를 쓰므로 DBSC와 무관합니다.

**Firefox는 DBSC를 공식적으로 도입하지 않기로 했습니다**(Mozilla `standards-positions` 저장소 `position: negative`). 다만 이건 "DBSC로는 안 막힌다"일 뿐이고, `gw.innogrid.com`은 별개 이유로 이미 Firefox에서도 파일 읽기가 안 됩니다 — `BIZCUBE_AT`/`HK`가 세션 쿠키라 **Firefox가 브라우저 실행 중엔 `cookies.sqlite`에 아예 쓰지 않는다**는 걸 실측으로 확인했습니다.

## 6. 크레덴셜이 안 잡힐 때 (문제 해결)

실행 시 `⚠️ 크레덴셜 미취득`이 뜨면, 에러 메시지에 **어떤 경로를 확인했는지**가 그대로 표시됩니다. 그걸 보고 아래처럼 처리하세요.

### 자주 걸리는 경우

- **Windows에서 아직 확장 프로그램을 안 썼다면** → 4번으로 가서 익스텐션을 설치하세요. 가장 확실합니다.
- **Ubuntu 등에서 Firefox가 snap/flatpak** → 프로필 경로가 표준(`~/.mozilla/firefox`)과 달라 못 찾습니다. 환경변수로 지정:
  ```sh
  # snap Firefox
  export INNO_CREED_FIREFOX_DIR=~/snap/firefox/common/.mozilla/firefox
  # flatpak Firefox
  export INNO_CREED_FIREFOX_DIR=~/.var/app/org.mozilla.firefox/.mozilla/firefox
  ```
- **Windows에서 Chrome/Edge 쿠키를 못 읽음(`os error 32` / 파일 사용 중)** → 최신 Chrome/Edge는 실행 중 쿠키 파일을 **배타적으로 잠급니다**. 완전히 종료한 뒤 다시 실행하면 파일은 읽히지만, `BIZCUBE_AT`/`HK`는 세션 쿠키라 그 시점엔 이미 사라져 있습니다(종료 전엔 잠겨서 못 읽고, 종료 후엔 쿠키가 없는 캐치-22) — **확장 프로그램(4번)이 유일하게 확실한 해법**입니다.
- **Chrome/Edge는 있는데 "복호화 실패"라고 나옴** →
  - **Linux 키링(gnome-keyring/kwallet, `v11`)**: 키링에서 키를 자동 조회하지만 **`secret-tool`이 필요**합니다. 없으면 설치 후 재시도: `sudo apt install libsecret-tools` (데스크톱 세션에서 키링이 잠금 해제돼 있어야 함).
  - **Windows Chrome/Edge app-bound(`v20`)**: 호출자 프로세스 경로를 검증하므로 inno-creed 같은 제3자 프로세스로는 **설계상 항상 거부**됩니다(버전·설정과 무관 — best-effort로 시도는 하지만 성공을 기대하지 마세요). 확장 프로그램(4번)을 쓰세요.
  - Firefox도 이 사이트에서는 세션 쿠키라 안 됩니다(위 DBSC 섹션 참고). 그래도 안 되면 아래 **크레덴셜 직접 지정**이 가장 확실합니다.

### 크레덴셜 직접 지정 (브라우저 읽기 우회)

브라우저에서 못 가져오는 환경이면 쿠키 값을 직접 넣습니다(모든 브라우저 읽기보다 우선). 브라우저 DevTools(F12) → **Application → Cookies → `https://gw.innogrid.com`** 에서 두 쿠키 값을 복사:

| 환경변수 | 값 |
|---|---|
| `INNO_CREED_AUTH_TOKEN` | `BIZCUBE_AT` 쿠키 값 (`%7C` 인코딩 그대로 가능) |
| `INNO_CREED_SIGN_KEY` | `BIZCUBE_HK` 쿠키 값 |

두 값 모두 있어야 사용됩니다. MCP로 실행 시 아래 `env` 블록에 넣으세요.

### 경로 오버라이드 환경변수

| 환경변수 | 용도 |
|---|---|
| `INNO_CREED_EXTENSION_CACHE` | 확장 프로그램 캐시 파일 경로(직접) — 기본값은 OS별 표준 로컬 데이터 디렉토리 |
| `INNO_CREED_FIREFOX_COOKIES` | Firefox `cookies.sqlite` 파일 경로(직접) |
| `INNO_CREED_FIREFOX_DIR` | Firefox 프로필 **디렉토리**(스캔) |
| `INNO_CREED_CHROME_COOKIES` | Chrome `Cookies` DB 파일 경로(직접) |
| `INNO_CREED_CHROME_USER_DATA` | Chrome `User Data` 루트 |
| `INNO_CREED_EDGE_COOKIES` | Edge `Cookies` DB 파일 경로(직접, Windows 전용) |
| `INNO_CREED_EDGE_USER_DATA` | Edge `User Data` 루트(Windows 전용) |
| `INNO_CREED_AUTH_TOKEN` | `BIZCUBE_AT` 값 직접 지정(브라우저 우회) |
| `INNO_CREED_SIGN_KEY` | `BIZCUBE_HK` 값 직접 지정(브라우저 우회) |

### ⚠️ MCP 클라이언트로 실행할 땐 `env`에 넣어야 합니다

Claude Code가 서버를 띄우면 셸의 `export`가 전달되지 않습니다. 등록 설정의 `env` 블록에 넣으세요:

```json
{
  "mcpServers": {
    "inno-creed": {
      "command": "/절대경로/inno-creed",
      "env": {
        "INNO_CREED_FIREFOX_DIR": "/home/you/snap/firefox/common/.mozilla/firefox"
      }
    }
  }
}
```

---

## 부록: 소스 빌드

프리빌트가 없는 환경(예: Intel 맥)이나 직접 빌드하려면:

```sh
git clone https://github.com/zilhak/inno-creed && cd inno-creed
cargo build --release        # → target/release/inno-creed (Windows는 inno-creed.exe)
```

**Rust 1.96+**(edition 2024, 번들 `libsqlite3-sys`가 최신 toolchain 요구)와 **C 컴파일러**(rusqlite 번들 SQLite 컴파일용)가 필요합니다.

`extension/`의 빌드 결과(`background.js`)는 저장소에 커밋돼 있어 4번 절차에는 별도 빌드가 필요 없습니다. 소스(`extension/src/background.ts`)를 고쳤다면 [Bun](https://bun.sh)으로 다시 빌드하세요:

```sh
cd extension
bun install
bun run build   # → background.js 갱신
```

---

전체 기능·안전 규약·동작 방식은 [README](../README.md)를 참고하세요.
