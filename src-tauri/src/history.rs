use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::error::AppError;
use crate::models::ParseResult;

const MAX_ENTRIES: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub title: String,
    pub raw_text: String,
    pub parse_result: ParseResult,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntrySummary {
    pub id: String,
    pub title: String,
    pub method: Option<String>,
    pub url: Option<String>,
    pub created_at: DateTime<Utc>,
}

fn history_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::InternalError(format!("Failed to get app data dir: {e}")))?;
    Ok(dir.join("history.json"))
}

fn load_store(app: &AppHandle) -> Result<Vec<HistoryEntry>, AppError> {
    let path = history_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&path)
        .map_err(|e| AppError::InternalError(format!("Failed to read history: {e}")))?;
    let entries: Vec<HistoryEntry> = serde_json::from_str(&data)
        .map_err(|e| AppError::InternalError(format!("Failed to parse history: {e}")))?;
    Ok(entries)
}

fn save_store(app: &AppHandle, entries: &[HistoryEntry]) -> Result<(), AppError> {
    let path = history_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::InternalError(format!("Failed to create dir: {e}")))?;
    }
    let data = serde_json::to_string_pretty(entries)
        .map_err(|e| AppError::InternalError(format!("Failed to serialize history: {e}")))?;
    fs::write(&path, data)
        .map_err(|e| AppError::InternalError(format!("Failed to write history: {e}")))?;
    Ok(())
}

fn generate_title(result: &ParseResult) -> String {
    if let (Some(method), Some(url)) = (&result.method, &result.url) {
        let path = url::Url::parse(url)
            .map(|u| u.path().to_string())
            .unwrap_or_else(|_| {
                // URL might be just a path like "/api/users"
                url.split('?').next().unwrap_or(url).to_string()
            });
        let title = format!("{method} {path}");
        if title.len() > 60 {
            format!("{}…", &title[..59])
        } else {
            title
        }
    } else {
        let now = Utc::now().format("%H:%M:%S");
        format!("Request at {now}")
    }
}

fn to_summary(entry: &HistoryEntry) -> HistoryEntrySummary {
    HistoryEntrySummary {
        id: entry.id.clone(),
        title: entry.title.clone(),
        method: entry.parse_result.method.clone(),
        url: entry.parse_result.url.clone(),
        created_at: entry.created_at,
    }
}

#[tauri::command]
pub fn history_save(
    app: AppHandle,
    raw_text: String,
    parse_result: ParseResult,
) -> Result<HistoryEntry, AppError> {
    let now = Utc::now();
    let entry = HistoryEntry {
        id: now.timestamp_millis().to_string(),
        title: generate_title(&parse_result),
        raw_text,
        parse_result,
        created_at: now,
    };

    let mut entries = load_store(&app)?;
    entries.insert(0, entry.clone());
    if entries.len() > MAX_ENTRIES {
        entries.truncate(MAX_ENTRIES);
    }
    save_store(&app, &entries)?;
    Ok(entry)
}

#[tauri::command]
pub fn history_list(app: AppHandle) -> Result<Vec<HistoryEntrySummary>, AppError> {
    let entries = load_store(&app)?;
    Ok(entries.iter().map(to_summary).collect())
}

#[tauri::command]
pub fn history_get(app: AppHandle, id: String) -> Result<HistoryEntry, AppError> {
    let entries = load_store(&app)?;
    entries
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| AppError::InternalError(format!("History entry not found: {id}")))
}

#[tauri::command]
pub fn history_rename(app: AppHandle, id: String, new_title: String) -> Result<(), AppError> {
    let mut entries = load_store(&app)?;
    if let Some(entry) = entries.iter_mut().find(|e| e.id == id) {
        entry.title = new_title;
        save_store(&app, &entries)?;
        Ok(())
    } else {
        Err(AppError::InternalError(format!(
            "History entry not found: {id}"
        )))
    }
}

#[tauri::command]
pub fn history_delete(app: AppHandle, id: String) -> Result<(), AppError> {
    let mut entries = load_store(&app)?;
    let len_before = entries.len();
    entries.retain(|e| e.id != id);
    if entries.len() == len_before {
        return Err(AppError::InternalError(format!(
            "History entry not found: {id}"
        )));
    }
    save_store(&app, &entries)?;
    Ok(())
}

#[tauri::command]
pub fn history_clear(app: AppHandle) -> Result<(), AppError> {
    save_store(&app, &[])?;
    Ok(())
}
