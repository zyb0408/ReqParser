use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::detector::is_http_like;

/// 剪贴板监听器的共享状态
pub struct ClipboardWatcherState {
    pub enabled: AtomicBool,
}

impl Default for ClipboardWatcherState {
    fn default() -> Self {
        Self {
            enabled: AtomicBool::new(false),
        }
    }
}

/// 启动剪贴板监听后台任务。在 app setup 时调用一次。
pub fn start_clipboard_watcher(app_handle: AppHandle, state: Arc<ClipboardWatcherState>) {
    tauri::async_runtime::spawn(async move {
        let mut last_clipboard = String::new();

        loop {
            if state.enabled.load(Ordering::Relaxed) {
                // 使用 spawn_blocking 包装 arboard 操作，
                // 因为 arboard::Clipboard 在 macOS 上不是 Send。
                let result = tauri::async_runtime::spawn_blocking(|| {
                    arboard::Clipboard::new().and_then(|mut cb| cb.get_text())
                })
                .await;

                if let Ok(Ok(content)) = result {
                    if content != last_clipboard && is_http_like(&content) {
                        let _ = app_handle.emit("clipboard-http-detected", &content);
                        last_clipboard = content;
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
}
