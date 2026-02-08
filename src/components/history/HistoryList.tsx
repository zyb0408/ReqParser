import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useApp } from "@/lib/app-context";
import { formatRelativeTime } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { History, Trash2, X } from "lucide-react";
import type { HistoryEntry, HistoryEntrySummary } from "@/types";

function HistoryItem({
  entry,
  onLoad,
  onDelete,
  onRename,
}: {
  entry: HistoryEntrySummary;
  onLoad: (id: string) => void;
  onDelete: (id: string) => void;
  onRename: (id: string, title: string) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [editTitle, setEditTitle] = useState(entry.title);

  const handleDoubleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setEditTitle(entry.title);
    setEditing(true);
  };

  const commitRename = () => {
    setEditing(false);
    const trimmed = editTitle.trim();
    if (trimmed && trimmed !== entry.title) {
      onRename(entry.id, trimmed);
    }
  };

  return (
    <div
      className="group flex items-start gap-3 px-4 py-3 hover:bg-muted/50 cursor-pointer transition-colors"
      onClick={() => onLoad(entry.id)}
    >
      {entry.method && (
        <Badge
          variant="secondary"
          className="shrink-0 mt-0.5 font-mono text-[10px] font-semibold bg-primary/10 text-primary border-none"
        >
          {entry.method}
        </Badge>
      )}

      <div className="flex-1 min-w-0">
        {editing ? (
          <input
            className="w-full text-sm bg-transparent border-b border-primary outline-none"
            value={editTitle}
            onChange={(e) => setEditTitle(e.target.value)}
            onBlur={commitRename}
            onKeyDown={(e) => {
              if (e.key === "Enter") commitRename();
              if (e.key === "Escape") setEditing(false);
            }}
            onClick={(e) => e.stopPropagation()}
            autoFocus
          />
        ) : (
          <span
            className="text-sm font-medium truncate block"
            onDoubleClick={handleDoubleClick}
          >
            {entry.title}
          </span>
        )}
        {entry.url && (
          <span className="text-xs text-muted-foreground truncate block mt-0.5">
            {entry.url}
          </span>
        )}
      </div>

      <span className="text-[11px] text-muted-foreground shrink-0 mt-0.5">
        {formatRelativeTime(entry.createdAt)}
      </span>

      <Button
        variant="ghost"
        size="icon"
        className="h-6 w-6 shrink-0 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive transition-opacity"
        onClick={(e) => {
          e.stopPropagation();
          onDelete(entry.id);
        }}
      >
        <Trash2 className="h-3 w-3" />
      </Button>
    </div>
  );
}

export function HistoryList() {
  const { state, dispatch } = useApp();
  const { historyList } = state;

  const handleLoad = async (id: string) => {
    try {
      const entry = await invoke<HistoryEntry>("history_get", { id });
      dispatch({
        type: "LOAD_FROM_HISTORY",
        payload: { rawText: entry.rawText, parseResult: entry.parseResult },
      });
    } catch {
      // load failed
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await invoke("history_delete", { id });
      dispatch({
        type: "SET_HISTORY_LIST",
        payload: historyList.filter((e) => e.id !== id),
      });
    } catch {
      // delete failed
    }
  };

  const handleRename = async (id: string, newTitle: string) => {
    try {
      await invoke("history_rename", { id, newTitle });
      dispatch({
        type: "SET_HISTORY_LIST",
        payload: historyList.map((e) =>
          e.id === id ? { ...e, title: newTitle } : e,
        ),
      });
    } catch {
      // rename failed
    }
  };

  const handleClearAll = async () => {
    try {
      await invoke("history_clear");
      dispatch({ type: "SET_HISTORY_LIST", payload: [] });
    } catch {
      // clear failed
    }
  };

  return (
    <div className="flex flex-col flex-1 min-h-0 animate-fade-up">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-border">
        <div className="flex items-center gap-2">
          <History className="h-4 w-4 text-muted-foreground" />
          <span className="text-sm font-medium">历史记录</span>
          {historyList.length > 0 && (
            <span className="text-xs text-muted-foreground">
              ({historyList.length})
            </span>
          )}
        </div>
        <div className="flex items-center gap-1">
          {historyList.length > 0 && (
            <Button
              variant="ghost"
              size="sm"
              className="h-7 px-2 text-xs text-muted-foreground hover:text-destructive"
              onClick={handleClearAll}
            >
              清空全部
            </Button>
          )}
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={() => dispatch({ type: "TOGGLE_HISTORY" })}
          >
            <X className="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>

      {/* List */}
      {historyList.length === 0 ? (
        <div className="flex flex-col items-center justify-center flex-1 text-muted-foreground">
          <History className="h-10 w-10 mb-3 opacity-30" />
          <span className="text-sm">暂无历史记录</span>
        </div>
      ) : (
        <div className="flex-1 overflow-y-auto divide-y divide-border/50">
          {historyList.map((entry) => (
            <HistoryItem
              key={entry.id}
              entry={entry}
              onLoad={handleLoad}
              onDelete={handleDelete}
              onRename={handleRename}
            />
          ))}
        </div>
      )}
    </div>
  );
}
