// gemini_companion.rs â€” Gemini WebView companion cho AI Brain (fusio-superapp)
//
// Pattern Ä‘Æ°á»£c port 100% tá»« app-biztada-ai-erp/src-tauri/src/lib.rs.
// Äiá»u khiá»ƒn báº±ng Rust + Tauri WebviewWindow â€” khÃ´ng dÃ¹ng Python, khÃ´ng dÃ¹ng Playwright.
//
// Kiáº¿n trÃºc:
//   - "gemini-companion-bg"    : WebviewWindow áº©n (headless) Ä‘á»ƒ gá»­i prompt, Ä‘á»c káº¿t quáº£
//   - "gemini-companion-login" : WebviewWindow hiá»‡n Ä‘á»ƒ ngÆ°á»i dÃ¹ng Ä‘Äƒng nháº­p Google
//                                â†’ tá»± Ä‘Ã³ng sau khi login thÃ nh cÃ´ng â†’ cháº¡y ngáº§m
//   - IPC qua history.replaceState() + window.url() fragment hash
//     Fragment format: #bzt-{KIND}-{stamp}-{url_encoded_payload}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs::OpenOptions, io::Write};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tokio::sync::oneshot;
use tokio::time::sleep;

const GEMINI_FLOW_REV: &str = "2026-05-03-r2";

/// Shared state: pending prompts awaiting JS response via receive_gemini_done
pub struct PendingPrompts(pub Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>);

impl Default for PendingPrompts {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }
}

/// Shared state: pending contexts awaiting JS response via receive_gemini_context
pub struct PendingContexts(pub Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>);

impl Default for PendingContexts {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }
}

// â”€â”€ Debug logging â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

pub struct GeminiLock(pub Arc<tokio::sync::Mutex<()>>);

impl Default for GeminiLock {
    fn default() -> Self {
        Self(Arc::new(tokio::sync::Mutex::new(())))
    }
}

pub fn append_gemini_debug(app: &AppHandle, message: &str) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!("[{ts}] {message}");

    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(dir) = app.path().app_data_dir() {
        paths.push(dir.join("gemini-auth-debug.log"));
    }
    let temp_dir = std::env::temp_dir().join("fusio-superapp");
    paths.push(temp_dir.join("gemini-auth-debug.log"));

    for p in &paths {
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(p) {
            let _ = writeln!(f, "{}", line);
        }
    }
}

// â”€â”€ Auth probe JS script â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// Inject vÃ o Gemini WebView â†’ ghi state vÃ o location.hash â†’ Rust Ä‘á»c láº¡i

fn build_gemini_auth_probe_script(stamp: &str, selectors: &crate::slm_client::webview_selectors::GeminiSelectors) -> String {
    let logged_in_json = serde_json::to_string(&selectors.logged_in).unwrap_or_else(|_| "[]".to_string());
    let sign_in_json = serde_json::to_string(&selectors.sign_in).unwrap_or_else(|_| "[]".to_string());
    
    format!(
        r#"(() => {{
    const mark = (state, detail) => {{
        const enc = encodeURIComponent(detail || '');
        const frag = `bzt-auth-{stamp}-${{state}}-${{enc}}`;
        try {{ history.replaceState(null, '', '#' + frag); }} catch {{ location.hash = frag; }}
    }};

    try {{
        const href = location.href || '';
        const host = location.hostname || '';
        const lowerHref = href.toLowerCase();

        if (host.includes('accounts.google.com') || lowerHref.includes('/signin') || lowerHref.includes('service=gemini')) {{
            mark('login_required', href || host);
            return;
        }}

        const loggedInSelectors = {logged_in_json};
        const signInSelectors = {sign_in_json};

        if (loggedInSelectors.some(s => {{ try {{ return !!document.querySelector(s); }} catch {{ return false; }} }})) {{
            mark('ready', href || host || 'gemini.google.com');
            return;
        }}

        if (signInSelectors.some(s => {{ try {{ return !!document.querySelector(s); }} catch {{ return false; }} }})) {{
            mark('login_required', href || host || 'gemini.google.com');
            return;
        }}

        mark('loading', href || host || 'unknown');
    }} catch (e) {{
        mark('loading', String(e));
    }}
}})();"#
    )
}

// â”€â”€ Auth state probe â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

pub async fn probe_gemini_auth_state(window: &WebviewWindow, selectors: &crate::slm_client::webview_selectors::GeminiSelectors) -> Result<(String, String), String> {
    let pre_url = window.url().map_err(|e| e.to_string())?.to_string();
    let pre_lower = pre_url.to_lowercase();
    if pre_lower.contains("accounts.google.com")
        || pre_lower.contains("/signin")
        || pre_lower.contains("service=gemini")
    {
        return Ok(("login_required".into(), pre_url));
    }

    let stamp = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis()
    );

    let script = build_gemini_auth_probe_script(&stamp, selectors);
    if let Err(e) = window.eval(&script) {
        return Ok((
            "loading".into(),
            format!("eval_failed:{e};url:{pre_url}"),
        ));
    }
    // Give WebView2 time to process history.replaceState
    sleep(Duration::from_millis(1400)).await;

    let url = window.url().map_err(|e| e.to_string())?.to_string();
    let marker = format!("#bzt-auth-{stamp}-");
    if let Some(rest) = url.split(&marker).nth(1) {
        let mut parts = rest.splitn(2, '-');
        let state = parts.next().unwrap_or("loading").to_string();
        let detail_raw = parts.next().unwrap_or("");
        let detail = urlencoding::decode(detail_raw)
            .map(|s| s.to_string())
            .unwrap_or_else(|_| detail_raw.to_string());
        if state != "loading" {
            return Ok((state, detail));
        }
    }

    let url = window.url().map_err(|e| e.to_string())?.to_string();
    let lower = url.to_lowercase();
    if lower.contains("accounts.google.com")
        || lower.contains("/signin")
        || lower.contains("service=gemini")
    {
        return Ok(("login_required".into(), url));
    }
    if lower.contains("gemini.google.com") {}
    Ok(("loading".into(), url))
}

// â”€â”€ Background hidden window â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

// -- BG_INIT_SCRIPT: injected into gemini-companion-bg once via initialization_script().
// MutationObserver watches DOM changes and invokes Tauri commands (push-based, zero polling).
// Runs before page content; survives SPA navigations.
fn build_bg_init_script(selectors: &crate::slm_client::webview_selectors::GeminiSelectors) -> String {
    let resp_sels_json = serde_json::to_string(&selectors.response_blocks).unwrap_or_else(|_| "[]".to_string());
    let stop_sels_json = serde_json::to_string(&selectors.stop_buttons).unwrap_or_else(|_| "[]".to_string());
    let send_sels_json = serde_json::to_string(&selectors.send_buttons).unwrap_or_else(|_| "[]".to_string());

    format!(r#"
(function () {{
  'use strict';
  var __bzt_stamp = '', __bzt_lastText = '', __bzt_doneSent = false, __bzt_doneTimer = null, __bzt_seenGen = false;

  function collectRoots() {{
    var roots = [document], seen = [document], queue = [document.documentElement];
    while (queue.length) {{
      var node = queue.shift();
      if (!node) continue;
      var kids = node.children ? Array.prototype.slice.call(node.children) : [];
      for (var i = 0; i < kids.length; i++) queue.push(kids[i]);
      if (node.shadowRoot && seen.indexOf(node.shadowRoot) === -1) {{
        seen.push(node.shadowRoot); roots.push(node.shadowRoot); queue.push(node.shadowRoot);
      }}
    }}
    return roots;
  }}

  function queryDeepAll(sels) {{
    var out = [], roots = collectRoots();
    for (var r = 0; r < roots.length; r++) {{
      for (var s = 0; s < sels.length; s++) {{
        try {{
          var f = roots[r].querySelectorAll
            ? Array.prototype.slice.call(roots[r].querySelectorAll(sels[s]))
            : [];
          for (var i = 0; i < f.length; i++) out.push(f[i]);
        }} catch (e) {{}}
      }}
    }}
    return out;
  }}

  var RESP_SELS = {resp_sels_json};
  var STOP_SELS = {stop_sels_json};
  var SEND_SELS = {send_sels_json};

  function getAnswerText() {{
    var blocks = queryDeepAll(RESP_SELS);
    for (var i = blocks.length - 1; i >= 0; i--) {{
        if (blocks[i].getAttribute('data-bzt-stamp') === __bzt_stamp) {{
            return (blocks[i].innerText || blocks[i].textContent || '').trim();
        }}
    }}
    return '';
  }}

  function getAnswerHtml() {{
    var blocks = queryDeepAll(RESP_SELS);
    for (var i = blocks.length - 1; i >= 0; i--) {{
        if (blocks[i].getAttribute('data-bzt-stamp') === __bzt_stamp) {{
            return (blocks[i].outerHTML || '').trim();
        }}
    }}
    return '';
  }}

  function isGenerating() {{
    var els = queryDeepAll(STOP_SELS);
    for (var i = 0; i < els.length; i++) {{
      var el = els[i];
      if (el && el.offsetParent !== null) {{
        var style = window.getComputedStyle(el);
        if (style.opacity !== '0' && style.visibility !== 'hidden' && style.display !== 'none') {{
           return true;
        }}
      }}
    }}
    return false;
  }}

  function safeInvoke(cmd, args) {{
    try {{
      var res = null;
      if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === 'function') {{
        res = window.__TAURI_INTERNALS__.invoke(cmd, args);
      }} else if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === 'function') {{
        res = window.__TAURI__.core.invoke(cmd, args);
      }} else if (typeof window.__TAURI_INVOKE__ === 'function') {{
        res = window.__TAURI_INVOKE__(cmd, args);
      }} else {{
        var payload = encodeURIComponent(JSON.stringify({{cmd: cmd, args: args}}));
        window.location.hash = "bzt-ipc-" + payload;
      }}
      
      if (res && typeof res.catch === 'function') {{
         res.catch(function(e) {{
            var payload = encodeURIComponent(JSON.stringify({{cmd: cmd, args: args}}));
            window.location.hash = "bzt-ipc-" + payload;
         }});
      }}
    }} catch (e) {{
        document.title = "[BZT] IPC ERROR: " + String(e);
    }}
  }}

  function emitDone(force = false) {{
    if (!__bzt_stamp || __bzt_doneSent) return;
    if (isGenerating() && !force) return;
    var text = getAnswerText();
    if (!text) return;
    __bzt_doneSent = true;
    var html = getAnswerHtml(), stamp = __bzt_stamp;
    __bzt_stamp = '';
    safeInvoke('receive_gemini_done', {{ stamp: stamp, html: html, text: text }});
  }}

  function onMutation() {{
    if (!__bzt_stamp) return;
    
    var bodyText = document.body.innerText || '';
    if (bodyText.includes('Lựa chọn A') || bodyText.includes('Choice A') || bodyText.includes('Option A') || bodyText.includes('Draft A')) {{
        var cardSelectors = [
            '[data-testid="response-option"]',
            '[data-testid="draft-option"]',
            '[data-testid="choice-option"]',
            'ms-response-option',
            '.response-option',
            '.draft-option'
        ];
        for (var i = 0; i < cardSelectors.length; i++) {{
            var el = document.querySelector(cardSelectors[i]);
            if (el) {{ el.click(); break; }}
        }}
    }}

    var blocks = queryDeepAll(RESP_SELS);
    for (var i = 0; i < blocks.length; i++) {{
        if (!blocks[i].hasAttribute('data-bzt-stamp')) {{
            blocks[i].setAttribute('data-bzt-stamp', __bzt_stamp);
        }}
    }}

    if (isGenerating()) {{
      __bzt_seenGen = true;
    }}
    
    var text = getAnswerText();
    if (text && text !== __bzt_lastText) {{
        __bzt_lastText = text;
        safeInvoke('receive_gemini_chunk', {{ stamp: __bzt_stamp, content: text }});
    }}

    if (__bzt_doneTimer) clearTimeout(__bzt_doneTimer);
    __bzt_doneTimer = setTimeout(emitDone, 3000);
    if (!isGenerating() && __bzt_seenGen && text && !__bzt_doneSent) {{
      clearTimeout(__bzt_doneTimer); __bzt_doneTimer = null; emitDone();
    }}
  }}

  window.__bzt_setStamp = function (stamp) {{
    __bzt_stamp    = stamp;
    __bzt_lastText = '';
    __bzt_seenGen  = false;
    __bzt_doneSent = false;
    
    var blocks = queryDeepAll(RESP_SELS);
    for (var i = 0; i < blocks.length; i++) {{
        if (!blocks[i].hasAttribute('data-bzt-stamp')) {{
            blocks[i].setAttribute('data-bzt-stamp', 'stale');
        }}
    }}
    if (__bzt_doneTimer) {{ clearTimeout(__bzt_doneTimer); __bzt_doneTimer = null; }}
  }};

  var observer = new MutationObserver(onMutation);
  function startObserving() {{
    if (document.body) {{
      observer.observe(document.body, {{ childList: true, subtree: true, characterData: true }});
    }}
  }}
  if (document.readyState === 'loading') {{
    document.addEventListener('DOMContentLoaded', startObserving);
  }} else {{
    startObserving();
  }}
}})();
"#)
}

pub fn ensure_bg_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(w) = app.get_webview_window("gemini-companion-bg") {
        return Ok(w);
    }

    let selectors = crate::slm_client::webview_selectors::load_selectors(app);
    let init_script = build_bg_init_script(&selectors.gemini);

    WebviewWindowBuilder::new(
        app,
        "gemini-companion-bg",
        WebviewUrl::External("https://gemini.google.com/app".parse().unwrap()),
    )
    .title("Gemini Companion")
    .initialization_script(&init_script)
    .visible(true)
    .resizable(true)
    .inner_size(1000.0, 800.0)
    .build()
    .map_err(|e| e.to_string())
}

// ── Auth marker persistence ───────────────────────────────────────────────────

fn persist_gemini_auth_marker(app: &AppHandle) -> Result<(), String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let marker = dir.join("gemini-auth-ok.flag");
    std::fs::write(marker, b"ok").map_err(|e| e.to_string())
}

/// Returns true nếu file marker tồn tại (đã đăng nhập ít nhất 1 lần).
pub fn gemini_has_auth_marker(app: &AppHandle) -> bool {
    if let Ok(dir) = app.path().app_data_dir() {
        return dir.join("gemini-auth-ok.flag").exists();
    }
    false
}

// ── Tauri command: ensure_gemini_login ────────────────────────────────────────
// Kiểm tra auth state trên bg window.
// Nếu chưa login → mở popup login window → đợi người dùng đăng nhập → tự đóng.
// Nếu đã login → trả về Ok ngay.

#[tauri::command]
pub async fn ensure_gemini_login(app: AppHandle) -> Result<(), String> {
    append_gemini_debug(&app, "ensure_gemini_login:start");
    let bg = ensure_bg_window(&app)?;
    let _ = bg.navigate("https://gemini.google.com/app".parse().unwrap());

    let selectors_config = crate::slm_client::webview_selectors::load_selectors(&app);
    // Probe bg window tối đa 6 lần (~18s) trước khi mở popup login
    let mut bg_ready = false;
    for attempt in 0u32..6 {
        sleep(Duration::from_millis(if attempt == 0 { 3000 } else { 1500 })).await;
        let (state, detail) = probe_gemini_auth_state(&bg, &selectors_config.gemini).await?;
        append_gemini_debug(
            &app,
            &format!(
                "ensure_gemini_login:bg_probe attempt={attempt} state={state} detail={detail}"
            ),
        );
        if state == "ready" {
            bg_ready = true;
            break;
        }
        if state == "login_required" {
            break;
        }
    }

    if bg_ready {
        if let Some(login_win) = app.get_webview_window("gemini-companion-login") {
            let _ = login_win.close();
        }
        let _ = persist_gemini_auth_marker(&app);
        append_gemini_debug(&app, "ensure_gemini_login:bg_ready_done");
        // Update brain AI config to mark gemini session
        // session ok
        return Ok(());
    }

    // Mở popup login window — người dùng đăng nhập thủ công
    let login = if let Some(existing) = app.get_webview_window("gemini-companion-login") {
        existing.show().map_err(|e| e.to_string())?;
        existing
    } else {
        WebviewWindowBuilder::new(
            &app,
            "gemini-companion-login",
            WebviewUrl::External("https://gemini.google.com/app".parse().unwrap()),
        )
        .title(&format!("Đăng nhập Gemini — FUSIO AI [{GEMINI_FLOW_REV}]"))
        .visible(true)
        .focused(true)
        .resizable(true)
        .inner_size(1280.0, 860.0)
        .build()
        .map_err(|e| e.to_string())?
    };

    // Đợi tối đa 10 phút loop 900ms
    let max_wait_ms = 10 * 60 * 1000;
    let mut waited = 0u64;
    loop {
        if app.get_webview_window("gemini-companion-login").is_none() {
            append_gemini_debug(&app, "ensure_gemini_login:login_window_closed_early");
            return Err("Cửa sổ đăng nhập đã bị đóng trước khi hoàn tất".into());
        }

        let (s, d) = probe_gemini_auth_state(&login, &selectors_config.gemini).await?;
        append_gemini_debug(
            &app,
            &format!("ensure_gemini_login:poll waited_ms={waited} state={s} detail={d}"),
        );
        if s == "ready" {
            let _ = login.close();
            let _ = bg.navigate("https://gemini.google.com/app".parse().unwrap());
            sleep(Duration::from_millis(1200)).await;
            let _ = persist_gemini_auth_marker(&app);
            // session ok
            append_gemini_debug(&app, "ensure_gemini_login:login_ready_closed");
            return Ok(());
        }

        if waited >= max_wait_ms {
            return Err("Hết thời gian chờ đăng nhập Gemini (10 phút)".into());
        }
        sleep(Duration::from_millis(900)).await;
        waited += 900;
    }
}

// ── Prompt injection JS script ────────────────────────────────────────────────

fn build_gemini_prompt_script(prompt: &str, image_base64: &Option<String>, stamp: &str, selectors: &crate::slm_client::webview_selectors::GeminiSelectors) -> String {
    let _prompt_json = serde_json::to_string(prompt).unwrap_or_else(|_| "\"\"".to_string());
    let encoded_prompt = urlencoding::encode(prompt);
    let stamp_json = serde_json::to_string(stamp).unwrap_or_else(|_| format!("\"{stamp}\""));
    let inputs_json = serde_json::to_string(&selectors.inputs).unwrap_or_else(|_| "[]".to_string());
    let btn_selectors_json = serde_json::to_string(&selectors.send_buttons).unwrap_or_else(|_| "[]".to_string());
    let image_json = match image_base64 {
        Some(b64) => format!("\"data:image/jpeg;base64,{}\"", b64),
        None => "null".to_string(),
    };

    format!(
        r#"(async function() {{
    const PROMPT = decodeURIComponent("{encoded_prompt}");
    const STAMP  = {stamp_json};
    const IMAGE_BASE64 = {image_json};

    const queryDeepAll = (selectors) => {{
        const out = [];
        const seen = new Set();
        const walk = (root) => {{
            if (!root) return;
            try {{
                for (const sel of selectors) {{
                    for (const el of root.querySelectorAll(sel)) {{
                        if (!seen.has(el)) {{ seen.add(el); out.push(el); }}
                    }}
                }}
                for (const el of root.querySelectorAll('*')) {{
                    if (el.shadowRoot) walk(el.shadowRoot);
                    for (const iframe of root.querySelectorAll('iframe')) {{
                        try {{ if (iframe.contentDocument) walk(iframe.contentDocument); }} catch {{}}
                    }}
                }}
            }} catch {{}}
        }};
        walk(document);
        return out;
    }};

    const isVisible = (el) => {{
        try {{
            if (!el) return false;
            const style = window.getComputedStyle(el);
            if (style.display === 'none' || style.visibility === 'hidden') return false;
            const rect = el.getBoundingClientRect();
            return rect.width > 0 && rect.height > 0;
        }} catch {{ return false; }}
    }};

    const isEditable = (el) => {{
        try {{
            if (!el) return false;
            if (el.isContentEditable) return true;
            if ('disabled' in el && el.disabled) return false;
            if ('readOnly' in el && el.readOnly) return false;
            return ('value' in el) || el.getAttribute('role') === 'textbox';
        }} catch {{ return false; }}
    }};

    const findInputCandidates = () => {{
        const selectors = {inputs_json};
        const seen = new Set();
        const out = [];
        for (const el of queryDeepAll(selectors)) {{
            if (!el || seen.has(el)) continue;
            seen.add(el);
            if (!isVisible(el) || !isEditable(el)) continue;
            out.push(el);
        }}
        return out;
    }};

    const fireInput = (el, value) => {{
        if (typeof el.click === 'function') el.click();
        if (typeof el.focus === 'function') el.focus();
        if ('value' in el) {{
            try {{
                const proto = el.tagName === 'TEXTAREA' ? window.HTMLTextAreaElement.prototype : window.HTMLInputElement.prototype;
                const setter = Object.getOwnPropertyDescriptor(proto, 'value')?.set;
                if (setter) setter.call(el, value);
                else el.value = value;
            }} catch {{ try {{ el.value = value; }} catch {{}} }}
            el.dispatchEvent(new Event('input', {{ bubbles: true }}));
            el.dispatchEvent(new Event('change', {{ bubbles: true }}));
            try {{ el.selectionStart = el.selectionEnd = value.length; }} catch {{}}
            return;
        }}
        if (el.isContentEditable) {{
            let success = false;
            try {{
                const selection = window.getSelection();
                const range = document.createRange();
                range.selectNodeContents(el);
                range.deleteContents();
                range.collapse(true);
                selection.removeAllRanges();
                selection.addRange(range);
                success = document.execCommand('insertText', false, value);
            }} catch {{}}
            if (!success) {{
                try {{ el.textContent = value; }} catch {{}}
                try {{ el.innerText = value; }} catch {{}}
                try {{ el.dispatchEvent(new InputEvent('beforeinput', {{ bubbles: true, data: value, inputType: 'insertText' }})); }} catch {{}}
                el.dispatchEvent(new InputEvent('input', {{ bubbles: true, data: value, inputType: 'insertText' }}));
                try {{ el.dispatchEvent(new Event('change', {{ bubbles: true }})); }} catch {{}}
            }}
        }}
    }};

    const currentInputValue = (el) => {{
        try {{
            if (!el) return '';
            if ('value' in el) return String(el.value || '');
            if (el.isContentEditable) return String(el.innerText || el.textContent || '');
        }} catch {{}}
        return '';
    }};

    const clickSend = () => {{
        const btnSelectors = {btn_selectors_json};
        for (const btn of queryDeepAll(btnSelectors)) {{
            if (btn && !btn.disabled && btn.getAttribute('aria-disabled') !== 'true') {{
                btn.click();
                return true;
            }}
        }}
        return false;
    }};

    const clickNewChat = () => {{
        const newChatSelectors = [
            'mat-icon[fonticon="gemini_chat_temp"]', 
            'button[aria-label*="New chat"]', 
            'button[aria-label*="Trò chuyện mới"]',
            '[data-testid="new-chat-button"]', 
            'mat-icon[fonticon="add"]'
        ];
        const icons = queryDeepAll(newChatSelectors);
        for (const icon of icons) {{
            if (icon.offsetParent !== null) {{
                const btn = icon.closest('button') || icon.closest('a') || icon;
                btn.click();
                return true;
            }}
        }}
        return false;
    }};

    const pasteImage = async (base64str, el) => {{
        try {{
            const res = await fetch(base64str);
            const blob = await res.blob();
            const file = new File([blob], "image.jpeg", {{ type: blob.type }});
            const dataTransfer = new DataTransfer();
            dataTransfer.items.add(file);
            const pasteEvent = new ClipboardEvent('paste', {{
                clipboardData: dataTransfer,
                bubbles: true,
                cancelable: true
            }});
            el.dispatchEvent(pasteEvent);
            return true;
        }} catch (e) {{ return false; }}
    }};

    const safeInvoke = (cmd, args) => {{
        try {{
            var res = null;
            if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === 'function') {{
                res = window.__TAURI_INTERNALS__.invoke(cmd, args);
            }} else if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === 'function') {{
                res = window.__TAURI__.core.invoke(cmd, args);
            }} else if (typeof window.__TAURI_INVOKE__ === 'function') {{
                res = window.__TAURI_INVOKE__(cmd, args);
            }} else {{
                var payload = encodeURIComponent(JSON.stringify({{cmd: cmd, args: args}}));
                window.location.hash = "bzt-ipc-" + payload;
            }}
            if (res && typeof res.catch === 'function') {{
                res.catch(function(e) {{
                    var payload = encodeURIComponent(JSON.stringify({{cmd: cmd, args: args}}));
                    window.location.hash = "bzt-ipc-" + payload;
                }});
            }}
        }} catch (e) {{
            document.title = "[BZT] IPC ERROR: " + String(e);
        }}
    }};

    try {{
        if (clickNewChat()) {{
            await new Promise(r => setTimeout(r, 500));
        }}

        const waitForInputs = (timeoutMs) => {{
            return new Promise(resolve => {{
                let els = findInputCandidates();
                if (els.length) return resolve(els);

                const observer = new MutationObserver(() => {{
                    els = findInputCandidates();
                    if (els.length) {{
                        observer.disconnect();
                        resolve(els);
                    }}
                }});

                observer.observe(document.body, {{ childList: true, subtree: true }});
                setTimeout(() => {{
                    observer.disconnect();
                    resolve([]);
                }}, timeoutMs);
            }});
        }};

        const inputs = await waitForInputs(5000);

        if (!inputs.length) {{
            safeInvoke('receive_gemini_log', {{ msg: '[bzt] gemini_input_not_found' }});
            console.error('[bzt] gemini_input_not_found');
            return;
        }}

        let input = null;
        let typed = '';
        for (const candidate of inputs) {{
            if (typeof candidate.click === 'function') candidate.click();
            if (typeof candidate.focus === 'function') candidate.focus();
            
            if (IMAGE_BASE64) {{
                const match = IMAGE_BASE64.match(/^data:(.*?);base64,/);
                const mime = match ? match[1] : 'application/octet-stream';
                let ext = 'jpeg';
                if (mime.includes('pdf')) ext = 'pdf';
                else if (mime.includes('word')) ext = 'docx';
                else if (mime.includes('excel')) ext = 'xlsx';
                
                const origClick = window.HTMLInputElement.prototype.click;
                let fileInjected = false;
                window.HTMLInputElement.prototype.click = async function() {{
                    if (this.type === 'file') {{
                        try {{
                            const res = await fetch(IMAGE_BASE64);
                            const blob = await res.blob();
                            const file = new File([blob], "upload." + ext, {{ type: blob.type }});
                            const dataTransfer = new DataTransfer();
                            dataTransfer.items.add(file);
                            this.files = dataTransfer.files;
                            this.dispatchEvent(new Event('change', {{ bubbles: true }}));
                            fileInjected = true;
                        }} catch(e) {{}}
                        window.HTMLInputElement.prototype.click = origClick;
                        return;
                    }}
                    origClick.call(this);
                }};

                const plusBtn = document.querySelector('mat-icon[data-mat-icon-name="plus"]');
                if (plusBtn) {{
                    const btn = plusBtn.closest('button') || plusBtn;
                    btn.click();
                    await new Promise(r => setTimeout(r, 500));
                    
                    const uploadBtn = document.querySelector('[data-test-id="local-images-files-uploader-button"]');
                    if (uploadBtn) {{
                        uploadBtn.click();
                        await new Promise(r => setTimeout(r, 800));
                    }}
                }}
                
                if (!fileInjected) {{
                    window.HTMLInputElement.prototype.click = origClick;
                    await pasteImage(IMAGE_BASE64, candidate);
                    await new Promise(r => setTimeout(r, 400));
                }}
            }}
            
            fireInput(candidate, PROMPT);
            typed = currentInputValue(candidate);
            if (typed && typed.trim()) {{
                input = candidate;
                break;
            }}
        }}

        if (!typed || !typed.trim()) {{
            safeInvoke('receive_gemini_log', {{ msg: '[bzt] gemini_input_fill_failed candidates=' + inputs.length }});
            console.error('[bzt] gemini_input_fill_failed candidates=' + inputs.length);
            return;
        }}

        // Register stamp BEFORE clicking send so MutationObserver starts watching
        if (typeof window.__bzt_setStamp === 'function') {{
            window.__bzt_setStamp(STAMP);
        }}

        // Wait 300ms for UI to update (React/Angular enables Send button)
        await new Promise(r => setTimeout(r, 300));

        let clicked = clickSend();
        if (!clicked) {{
            // Try again after 500ms
            await new Promise(r => setTimeout(r, 500));
            clicked = clickSend();
        }}
    }} catch (e) {{
        safeInvoke('receive_gemini_log', {{ msg: '[bzt] prompt_error: ' + String(e) }});
        console.error('[bzt] prompt_error:', String(e));
    }}
}})();"#
    )
}

// Gửi prompt lên Gemini bg window, đợi kết quả qua fragment polling.
// Frontend gọi invoke('run_gemini_background_prompt', { prompt })

#[tauri::command]
pub async fn run_gemini_background_prompt(
    app: AppHandle,
    prompt: String,
    image_base64: Option<String>,
    task_id: Option<String>,
) -> Result<String, String> {
    append_gemini_debug(
        &app,
        &format!("run_gemini_background_prompt:start len={}", prompt.len()),
    );
    
    // 1. Acquire the native Mutex lock to enforce strictly sequential execution in Rust
    let lock_state = app.state::<GeminiLock>();
    let _guard = lock_state.0.lock().await;
    
    let bg = ensure_bg_window(&app)?;

    let selectors_config = crate::slm_client::webview_selectors::load_selectors(&app);

    // Fast-path: if auth marker exists, probe once instead of full login loop (up to 18s)
    let has_marker = gemini_has_auth_marker(&app);
    if has_marker {
        let (state, _) = probe_gemini_auth_state(&bg, &selectors_config.gemini).await?;
        append_gemini_debug(&app, &format!("run_gemini_background_prompt:fast_probe state={state}"));
        if state == "login_required" {
            append_gemini_debug(&app, "run_gemini_background_prompt:session_expired_fallback");
            ensure_gemini_login(app.clone()).await?;
        } else if state == "loading" {
            sleep(Duration::from_millis(2000)).await;
            let (state2, _) = probe_gemini_auth_state(&bg, &selectors_config.gemini).await?;
            if state2 == "login_required" {
                ensure_gemini_login(app.clone()).await?;
            }
        }
        // state == "ready" -> proceed without re-navigate
    } else {
        // No auth marker -> need first-time login
        ensure_gemini_login(app.clone()).await?;
    }

    // Only navigate if bg window is not already on gemini.google.com
    let current_url = bg.url().map_err(|e| e.to_string())?.to_string();
    let need_navigate = !current_url.contains("gemini.google.com");
    if need_navigate {
        let _ = bg.navigate("https://gemini.google.com/app".parse().unwrap());
        append_gemini_debug(&app, "run_gemini_background_prompt:navigate_bg");
        sleep(Duration::from_millis(1800)).await;
    } else {
        sleep(Duration::from_millis(300)).await;
    }

    // Thử lại tối đa 5 lần nếu timeout
    let mut attempt = 1;
    let max_retries = 5;
    let mut retry_delay_secs = 5;
    let mut response_result = Err("Khởi tạo thất bại".into());

    let base_stamp = task_id.ok_or_else(|| {
        "SSOT Violation: Missing task_id. Caller MUST provide an explicit DAG Node ID.".to_string()
    })?;

    while attempt <= max_retries {
        let stamp = format!("{}-{}", base_stamp, attempt);

        // Register oneshot channel BEFORE eval so JS MutationObserver can call receive_gemini_done
        let (tx, mut rx) = tokio::sync::oneshot::channel::<String>();
        {
            let pending = app.state::<PendingPrompts>();
            let mut map = pending.0.lock().map_err(|e| e.to_string())?;
            map.insert(stamp.clone(), tx);
        }

        let script = build_gemini_prompt_script(&prompt, &image_base64, &stamp, &selectors_config.gemini);
        if let Err(e) = bg.eval(&script) {
            let pending = app.state::<PendingPrompts>();
            let mut map = pending.0.lock().unwrap_or_else(|e2| e2.into_inner());
            map.remove(&stamp);
            return Err(e.to_string());
        }
        append_gemini_debug(
            &app,
            &format!("run_gemini_background_prompt:eval_ok stamp={stamp} attempt={attempt}"),
        );

        let last_activity = std::sync::Arc::new(tokio::sync::Mutex::new(std::time::Instant::now()));

        // Fallback polling task: reads title for errors and URL hash for IPC data!
        let bg_clone = bg.clone();
        let app_clone = app.clone();
        let stamp_clone = stamp.clone();
        let activity_clone = last_activity.clone();
        tokio::spawn(async move {
            for _ in 0..1200 {
                tokio::time::sleep(Duration::from_millis(100)).await;
                
                // Exit early if the stamp has already been resolved/removed by standard IPC
                {
                    let pending = app_clone.state::<PendingPrompts>();
                    let map_contains = {
                        let map = pending.0.lock().unwrap_or_else(|e| e.into_inner());
                        map.contains_key(&stamp_clone)
                    };
                    if !map_contains {
                        break;
                    }
                }
                
                // 1. Check title for IPC errors
                if let Ok(title) = bg_clone.title() {
                    if title.starts_with("[BZT]") {
                        append_gemini_debug(&app_clone, &format!("WEBVIEW_TITLE_LOG: {}", title));
                    }
                }
                
                // 2. Check URL hash for fallback IPC
                if let Ok(url) = bg_clone.url() {
                    let url_str = url.to_string();
                    if let Some(idx) = url_str.find("#bzt-ipc-") {
                        let encoded = &url_str[idx + 9..];
                        if let Ok(decoded) = urlencoding::decode(encoded) {
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&decoded) {
                                if let Some(cmd) = parsed.get("cmd").and_then(|v| v.as_str()) {
                                    if cmd == "receive_gemini_done" {
                                        if let Some(args) = parsed.get("args") {
                                            if let (Some(s), Some(text)) = (args.get("stamp").and_then(|v| v.as_str()), args.get("text").and_then(|v| v.as_str())) {
                                                append_gemini_debug(&app_clone, &format!("HASH_POLL: receive_gemini_done: {}", s));
                                                let pending = app_clone.state::<PendingPrompts>();
                                                let mut map = pending.0.lock().unwrap_or_else(|e| e.into_inner());
                                                if let Some(tx) = map.remove(s) {
                                                    let _ = tx.send(text.to_string());
                                                }
                                                let _ = bg_clone.eval("window.location.hash = '';");
                                                break;
                                            }
                                        }
                                    } else if cmd == "receive_gemini_chunk" {
                                        if let Some(args) = parsed.get("args") {
                                            if let (Some(_s), Some(content)) = (args.get("stamp").and_then(|v| v.as_str()), args.get("content").and_then(|v| v.as_str())) {
                                                let _ = app_clone.emit("gemini-chunk", serde_json::json!({ "content": content }));
                                                let mut act = activity_clone.lock().await;
                                                *act = std::time::Instant::now();
                                                let _ = bg_clone.eval("window.location.hash = '';");
                                            }
                                        }
                                    } else if cmd == "receive_gemini_log" {
                                        if let Some(args) = parsed.get("args") {
                                            if let Some(msg) = args.get("msg").and_then(|v| v.as_str()) {
                                                append_gemini_debug(&app_clone, &format!("HASH_POLL_LOG: {}", msg));
                                                let _ = bg_clone.eval("window.location.hash = '';");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        // Loop to wait for result or 10s idle timeout
        let mut timed_out = false;
        loop {
            match tokio::time::timeout(Duration::from_millis(500), &mut rx).await {
                Ok(Ok(html)) => {
                    append_gemini_debug(
                        &app,
                        &format!("run_gemini_background_prompt:done chars={}", html.len()),
                    );
                    response_result = Ok(html);
                    break;
                }
                Ok(Err(_)) => {
                    response_result = Err("Gemini IPC channel closed unexpectedly".into());
                    break;
                }
                Err(_) => {
                    // Check if idle for more than 45 seconds
                    let elapsed = last_activity.lock().await.elapsed();
                    if elapsed.as_secs() > 45 {
                        timed_out = true;
                        let _ = bg.clone().eval("if (window.__bzt_setStamp) window.__bzt_setStamp('');");
                        response_result = Err("Gemini Webview Timeout: No response for 45 seconds".into());
                        break;
                    }
                }
            }
        }

        if !timed_out && response_result.is_ok() {
            break;
        }

        if timed_out {
            let pending = app.state::<PendingPrompts>();
            let mut map = pending.0.lock().unwrap_or_else(|e| e.into_inner());
            map.remove(&stamp);
            append_gemini_debug(&app, &format!("run_gemini_background_prompt:timeout_idle attempt={attempt}"));
            response_result = Err("Gemini timeout (45s không có phản hồi)".into());
        }

        if response_result.is_err() && attempt < max_retries {
            append_gemini_debug(&app, &format!("run_gemini_background_prompt:retry attempt={}", attempt + 1));
            // Không reload trang, chỉ đợi và vòng lặp sẽ build lại script click New Chat -> Paste -> Gửi
            sleep(Duration::from_secs(retry_delay_secs)).await;
            retry_delay_secs += 5;
        }
        attempt += 1;
    }

    // Reset to New Chat sau khi trả kết quả (để tránh nhiễu context cho prompt tiếp theo)
    // Giống pattern wkr-ai-controller
    let _ = bg.navigate("https://gemini.google.com/app".parse().unwrap());

    response_result
}

// â”€â”€ Tauri command: get_gemini_debug_log â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

// ── Tauri command: parse_jd_to_profile ──────────────────────────────────────────
// Tự động phân tích Job Description thành JSON MatchingProfile
#[tauri::command]
pub async fn parse_jd_to_profile(
    app: AppHandle,
    jd_text: String,
) -> Result<String, String> {
    let prompt = format!(
        "Bạn là một chuyên gia nhân sự. Hãy phân tích đoạn Job Description (JD) sau và trả về DUY NHẤT một chuỗi JSON chuẩn có cấu trúc như sau (KHÔNG giải thích, KHÔNG markdown):\n\
        {{\n\
            \"tech_stack\": [{{\"name\": \"Rust\", \"weight\": 1.0}}, {{\"name\": \"Tauri\", \"weight\": 0.8}}],\n\
            \"domain_knowledge\": [{{\"name\": \"Blockchain\", \"weight\": 0.5}}],\n\
            \"seniority_level\": \"Senior\",\n\
            \"work_model\": \"Remote\",\n\
            \"min_salary\": 0,\n\
            \"max_salary\": 5000\n\
        }}\n\
        \n\
        JD:\n{}",
        jd_text
    );
    // Sử dụng UUID để định danh task_id (giả lập ở đây)
    let stamp = format!("parse-jd-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
    let html = run_gemini_background_prompt(app, prompt, None, Some(stamp)).await?;
    
    // Bóc tách JSON từ HTML
    if let Some(start) = html.find("{") {
        if let Some(end) = html.rfind("}") {
            return Ok(html[start..=end].to_string());
        }
    }
    Err("Không tìm thấy cấu trúc JSON trong phản hồi của Gemini".to_string())
}

// ── Tauri command: parse_cv_to_profile ──────────────────────────────────────────
// Tự động phân tích CV hoặc DNA Profile thành JSON MatchingProfile
#[tauri::command]
pub async fn parse_cv_to_profile(
    app: AppHandle,
    cv_text: String,
) -> Result<String, String> {
    let prompt = format!(
        "Bạn là một AI phân tích năng lực. Hãy phân tích đoạn thông tin ứng viên/CV sau và trả về DUY NHẤT một chuỗi JSON chuẩn có cấu trúc như sau (KHÔNG giải thích, KHÔNG markdown):\n\
        {{\n\
            \"tech_stack\": [{{\"name\": \"React\", \"weight\": 1.0}}],\n\
            \"domain_knowledge\": [{{\"name\": \"FinTech\", \"weight\": 0.8}}],\n\
            \"seniority_level\": \"Mid\",\n\
            \"work_model\": \"Remote\",\n\
            \"min_salary\": 2000,\n\
            \"max_salary\": 0\n\
        }}\n\
        \n\
        CV/Profile:\n{}",
        cv_text
    );
    let stamp = format!("parse-cv-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
    let html = run_gemini_background_prompt(app, prompt, None, Some(stamp)).await?;
    
    if let Some(start) = html.find("{") {
        if let Some(end) = html.rfind("}") {
            return Ok(html[start..=end].to_string());
        }
    }
    Err("Không tìm thấy cấu trúc JSON trong phản hồi của Gemini".to_string())
}

// -- Tauri command: receive_gemini_chunk
// Called by MutationObserver JS when streaming partial text is detected.
// Emits "gemini-chunk" event → frontend brainStore listener updates placeholder message.

#[tauri::command]
pub async fn receive_gemini_chunk(
    app: AppHandle,
    stamp: String,
    content: String,
) -> Result<(), String> {
    let _ = app.emit("gemini-chunk", serde_json::json!({ "content": content }));
    let _ = stamp; // stamp reserved for future per-request filtering
    Ok(())
}

// -- Tauri command: receive_gemini_done
// Called by MutationObserver JS when generation is complete.
// Resolves the oneshot channel in PendingPrompts → run_gemini_background_prompt returns.

#[tauri::command]
pub async fn receive_gemini_done(
    app: AppHandle,
    pending: tauri::State<'_, PendingPrompts>,
    stamp: String,
    html: String,
    text: String,
) -> Result<(), String> {
    append_gemini_debug(
        &app,
        &format!("receive_gemini_done: stamp={stamp} html_len={}", html.len()),
    );
    let mut map = pending.0.lock().map_err(|e| e.to_string())?;
    if let Some(tx) = map.remove(&stamp) {
        let result = if text.is_empty() { html } else { text };
        let _ = tx.send(result);
    }
    Ok(())
}

#[tauri::command]
pub async fn receive_gemini_log(
    app: AppHandle,
    msg: String,
) -> Result<(), String> {
    append_gemini_debug(&app, &format!("webview_js_log: {}", msg));
    Ok(())
}

#[tauri::command]
pub fn get_gemini_debug_log(app: AppHandle, lines: Option<u32>) -> Result<String, String> {
    let max_lines = usize::try_from(lines.unwrap_or(200).min(2000)).unwrap_or(200);
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(dir) = app.path().app_data_dir() {
        candidates.push(dir.join("gemini-auth-debug.log"));
    }
    candidates.push(
        std::env::temp_dir()
            .join("fusio-superapp")
            .join("gemini-auth-debug.log"),
    );

    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let all_lines: Vec<&str> = content.lines().collect();
            let start = all_lines.len().saturating_sub(max_lines);
            return Ok(all_lines[start..].join("\n"));
        }
    }

    Err("Gemini debug log not found".into())
}

// â”€â”€ Tauri command: extract_gemini_context â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// TrÃ­ch xuáº¥t ngá»¯ cáº£nh (cookie, localStorage, v.v) cá»§a webview Gemini.
// DÃ¹ng Ä‘á»ƒ truyá» n vÃ o cho Antigravity MCP Server (hoáº·c Playwright headless).

#[tauri::command]
pub async fn extract_gemini_context(app: AppHandle) -> Result<String, String> {
    append_gemini_debug(&app, "extract_gemini_context:start");
    let bg = ensure_bg_window(&app)?;

    // Fast fail if not logged in
    let has_marker = gemini_has_auth_marker(&app);
    if !has_marker {
        return Err("Vui lÃ²ng Ä‘Äƒng nháº­p Gemini trong tÃ¹y chá» n trÆ°á»›c khi sá» dá»¥ng cÃ´ng cá»¥ tÃ¬m kiáº¿m má»Ÿ rá»™ng.".into());
    }

    let stamp = format!(
        "ctx-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        uuid::Uuid::new_v4().to_string().replace("-", "")[..6].to_string()
    );

    let (tx, rx) = oneshot::channel::<String>();
    {
        let pending = app.state::<PendingContexts>();
        let mut map = pending.0.lock().map_err(|e| e.to_string())?;
        map.insert(stamp.clone(), tx);
    }

    let script = format!(r#"
        (() => {{
            try {{
                const context = {{
                    cookie: document.cookie,
                    url: window.location.href,
                    localStorage: JSON.stringify(window.localStorage)
                }};
                if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {{
                    window.__TAURI_INTERNALS__.invoke('receive_gemini_context', {{ stamp: '{}', context: JSON.stringify(context) }});
                }}
            }} catch(e) {{
                if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {{
                    window.__TAURI_INTERNALS__.invoke('receive_gemini_context', {{ stamp: '{}', context: JSON.stringify({{error: e.toString()}}) }});
                }}
            }}
        }})();
    "#, stamp, stamp);

    if let Err(e) = bg.eval(&script) {
        let pending = app.state::<PendingContexts>();
        let mut map = pending.0.lock().unwrap();
        map.remove(&stamp);
        return Err(e.to_string());
    }

    match tokio::time::timeout(Duration::from_secs(10), rx).await {
        Ok(Ok(ctx)) => {
            append_gemini_debug(&app, "extract_gemini_context:success");
            Ok(ctx)
        },
        Ok(Err(_)) => Err("Gemini Context Channel closed unexpectedly".into()),
        Err(_) => Err("Timeout reading Webview Context".into()),
    }
}

#[tauri::command]
pub async fn receive_gemini_context(
    pending: tauri::State<'_, PendingContexts>,
    stamp: String,
    context: String,
) -> Result<(), String> {
    let mut map = pending.0.lock().unwrap();
    if let Some(tx) = map.remove(&stamp) {
        let _ = tx.send(context);
    }
    Ok(())
}


// â”€â”€ Tauri command: gemini_has_session â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// Frontend poll Ä‘á»ƒ biáº¿t Gemini Ä‘Ã£ Ä‘Äƒng nháº­p hay chÆ°a (khÃ´ng cáº§n má»Ÿ browser)


// ── Tauri command: warm_up_gemini_bg ─────────────────────────────────────────
// Khởi tạo bg WebView window sớm + navigate đến Gemini, KHÔNG probe auth.
// Gọi fire-and-forget ngay khi detect gemini_browser mode để warm up sớm.

#[tauri::command]
pub async fn warm_up_gemini_bg(app: AppHandle) -> Result<(), String> {
    append_gemini_debug(&app, "warm_up_gemini_bg:start");
    let bg = ensure_bg_window(&app)?;
    let current_url = bg.url().map_err(|e| e.to_string())?.to_string();
    if !current_url.contains("gemini.google.com") {
        let _ = bg.navigate("https://gemini.google.com/app".parse().unwrap());
        append_gemini_debug(&app, "warm_up_gemini_bg:navigated");
    } else {
        append_gemini_debug(&app, "warm_up_gemini_bg:already_on_gemini");
    }
    Ok(())
}

#[tauri::command]
pub fn gemini_has_session(app: AppHandle) -> bool {
    gemini_has_auth_marker(&app)
}

/// Đăng xuất phiên Gemini hiện tại rồi mở login window để đổi tài khoản Google.
/// KHÔNG reset Brain AI config (mode vẫn là gemini_browser).
/// Luồng: xóa auth marker → đóng bg window → mở login window như ensure_gemini_login.
#[tauri::command]
pub async fn gemini_switch_account(app: AppHandle) -> Result<(), String> {
    append_gemini_debug(&app, "gemini_switch_account:start");

    // 1. Xóa auth marker file
    if let Ok(dir) = app.path().app_data_dir() {
        let marker = dir.join("gemini-auth-ok.flag");
        let _ = std::fs::remove_file(&marker);
    }

    // 2. Cập nhật brain config: mark session invalid (giữ mode = gemini_browser)
    // session cleared

    // 3. Đóng bg window — sẽ được tạo lại fresh khi ensure_gemini_login chạy
    if let Some(bg) = app.get_webview_window("gemini-companion-bg") {
        let _ = bg.close();
    }

    // 4. Mở login window để người dùng chọn / đăng nhập tài khoản Google khác
    append_gemini_debug(&app, "gemini_switch_account:opening_login");
    ensure_gemini_login(app).await
}
