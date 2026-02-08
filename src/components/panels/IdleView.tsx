import { invoke } from "@tauri-apps/api/core";
import { useApp } from "@/lib/app-context";
import { useParse } from "@/lib/use-parse";
import { formatRelativeTime } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Play, Loader2, Terminal, ChevronRight } from "lucide-react";
import type { HistoryEntry } from "@/types";

const PLACEHOLDER = `GET /api/v1/users?page=1&limit=20 HTTP/1.1
Host: api.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
Content-Type: application/json
Accept: application/json`;

export function IdleView() {
  const { state, dispatch } = useApp();
  const { parse } = useParse();

  const handlePaste = async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (!text.trim()) return;
      dispatch({ type: "SET_RAW_TEXT", payload: text });
      parse(text);
    } catch {
      // clipboard read failed
    }
  };

  const handleLoadHistory = async (id: string) => {
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

  const lineCount = state.rawText ? state.rawText.split("\n").length : 0;
  const hasText = state.rawText.trim().length > 0;
  const isParsing = state.parseState === "parsing";
  const recentHistory = state.historyList.slice(0, 5);

  return (
    <div className="flex flex-col items-center justify-center flex-1 px-6 animate-fade-up">
      {/* Brand */}
      <div className="flex items-center gap-3 mb-8">
        <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-primary/10">
          <Terminal className="h-5 w-5 text-primary" />
        </div>
        <div>
          <h1 className="text-xl font-semibold tracking-tight">ReqParser</h1>
          <p className="text-sm text-muted-foreground">粘贴 HTTP 请求，即刻解析</p>
        </div>
      </div>

      {/* Textarea */}
      <div className="w-full max-w-3xl textarea-glow rounded-xl border border-border bg-card transition-shadow duration-200">
        <textarea
          className="w-full h-64 p-4 font-mono text-sm leading-relaxed bg-transparent border-none resize-none focus:outline-none placeholder:text-[var(--text-dimmed)]"
          value={state.rawText}
          onChange={(e) => dispatch({ type: "SET_RAW_TEXT", payload: e.target.value })}
          placeholder={PLACEHOLDER}
          spellCheck={false}
          autoFocus
        />
        {/* Inline status bar */}
        <div className="flex items-center justify-between px-4 py-2 border-t border-border/50 text-[11px] text-[var(--text-dimmed)]">
          <span>{hasText ? `${lineCount} 行` : "粘贴或输入 HTTP 文本"}</span>
          <span className="font-mono">⌘+Enter 解析</span>
        </div>
      </div>

      {/* Actions */}
      <div className="flex items-center gap-3 mt-6">
        <Button
          variant="outline"
          size="sm"
          className="h-9 px-4"
          onClick={handlePaste}
          disabled={isParsing}
        >
          粘贴并解析
        </Button>
        {hasText && (
          <Button
            size="sm"
            className="h-9 px-4"
            onClick={() => parse()}
            disabled={isParsing}
          >
            {isParsing ? (
              <Loader2 className="h-4 w-4 animate-spin mr-1.5" />
            ) : (
              <Play className="h-4 w-4 mr-1.5" />
            )}
            解析
          </Button>
        )}
      </div>

      {/* Recent history */}
      {recentHistory.length > 0 && !state.historyOpen && (
        <div className="w-full max-w-3xl mt-8">
          <div className="flex items-center justify-between mb-2 px-1">
            <span className="text-xs font-medium text-muted-foreground">最近解析</span>
            <button
              className="flex items-center gap-0.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
              onClick={() => dispatch({ type: "TOGGLE_HISTORY" })}
            >
              查看全部
              <ChevronRight className="h-3 w-3" />
            </button>
          </div>
          <div className="rounded-lg border border-border/50 bg-card/50 divide-y divide-border/50">
            {recentHistory.map((entry) => (
              <button
                key={entry.id}
                className="w-full flex items-center gap-3 px-3 py-2.5 text-left hover:bg-muted/50 transition-colors first:rounded-t-lg last:rounded-b-lg"
                onClick={() => handleLoadHistory(entry.id)}
              >
                {entry.method && (
                  <Badge
                    variant="secondary"
                    className="shrink-0 font-mono text-[10px] font-semibold bg-primary/10 text-primary border-none"
                  >
                    {entry.method}
                  </Badge>
                )}
                <span className="text-sm truncate flex-1 min-w-0">{entry.title}</span>
                <span className="text-[11px] text-muted-foreground shrink-0">
                  {formatRelativeTime(entry.createdAt)}
                </span>
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
