// src/background.ts
var HOST_NAME = "com.innogrid.inno_creed";
var DOMAIN = "gw.innogrid.com";
var COOKIE_NAMES = ["BIZCUBE_AT", "BIZCUBE_HK"];
var syncTimer;
function matchesDomain(domain) {
  const bare = domain.startsWith(".") ? domain.slice(1) : domain;
  return bare === DOMAIN;
}
function sendNative(message) {
  const p = chrome.runtime.connectNative(HOST_NAME);
  let responded = false;
  p.onMessage.addListener((response) => {
    responded = true;
    if (!response?.ok) {
      console.warn("[inno-creed] native host 처리 실패:", response?.error);
    }
  });
  p.onDisconnect.addListener(() => {
    const err = chrome.runtime.lastError;
    if (!responded) {
      console.warn("[inno-creed] native host 연결 실패:", err?.message);
    }
  });
  try {
    p.postMessage(message);
  } catch (e) {
    console.warn("[inno-creed] native host 전송 실패:", e);
  }
}
async function syncCookies() {
  const cookies = await chrome.cookies.getAll({ domain: DOMAIN });
  const authTokenCookie = cookies.find((c) => c.name === "BIZCUBE_AT");
  const signKeyCookie = cookies.find((c) => c.name === "BIZCUBE_HK");
  if (!authTokenCookie || !signKeyCookie) {
    console.log(`[inno-creed] ${DOMAIN} 쿠키 ${cookies.length}개 중 BIZCUBE_AT/HK 없음 — 미로그인 상태로 보임`);
    return;
  }
  console.log("[inno-creed] BIZCUBE_AT/HK 발견, native host로 전달");
  sendNative({ authToken: authTokenCookie.value, signKey: signKeyCookie.value });
}
function scheduleSync() {
  if (syncTimer)
    clearTimeout(syncTimer);
  syncTimer = setTimeout(syncCookies, 500);
}
chrome.cookies.onChanged.addListener((changeInfo) => {
  const c = changeInfo.cookie;
  if (!matchesDomain(c.domain) || !COOKIE_NAMES.includes(c.name))
    return;
  if (changeInfo.removed) {
    sendNative({ clear: true });
  } else {
    scheduleSync();
  }
});
syncCookies();
