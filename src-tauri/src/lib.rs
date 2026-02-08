mod clipboard;
mod curl_parser;
mod detector;
mod error;
mod fetch_parser;
mod models;
mod parser;

use std::sync::atomic::Ordering;
use std::sync::Arc;

use tauri::State;

use clipboard::ClipboardWatcherState;
use detector::InputFormat;
use error::AppError;
use models::ParseResult;

/// 解析 HTTP 文本，自动检测输入格式（cURL / fetch / 原始 HTTP）。
#[tauri::command]
fn parse_text(raw_text: String) -> Result<ParseResult, AppError> {
    if raw_text.trim().is_empty() {
        return Err(AppError::ParseError("Input text is empty".to_string()));
    }
    let result = match detector::detect_input_format(&raw_text) {
        InputFormat::Curl => curl_parser::parse_curl(&raw_text),
        InputFormat::Fetch => fetch_parser::parse_fetch(&raw_text),
        InputFormat::RawHttp => parser::parse_http_text(&raw_text),
        InputFormat::Unknown => parser::parse_http_text(&raw_text),
    };
    Ok(result)
}

/// 检测文本是否像 HTTP 数据。
#[tauri::command]
fn check_http_like(text: String) -> bool {
    detector::is_http_like(&text)
}

/// 开关剪贴板监听。传入 enabled 强制设置，不传则切换。
#[tauri::command]
fn toggle_clipboard_watcher(
    state: State<'_, Arc<ClipboardWatcherState>>,
    enabled: Option<bool>,
) -> bool {
    let new_value = enabled.unwrap_or_else(|| !state.enabled.load(Ordering::Relaxed));
    state.enabled.store(new_value, Ordering::Relaxed);
    new_value
}

/// 获取剪贴板监听状态。
#[tauri::command]
fn get_clipboard_watcher_status(state: State<'_, Arc<ClipboardWatcherState>>) -> bool {
    state.enabled.load(Ordering::Relaxed)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let watcher_state = Arc::new(ClipboardWatcherState::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(watcher_state.clone())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            clipboard::start_clipboard_watcher(app_handle, watcher_state.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            parse_text,
            check_http_like,
            toggle_clipboard_watcher,
            get_clipboard_watcher_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
