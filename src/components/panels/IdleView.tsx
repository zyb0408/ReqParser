import { useApp } from "@/lib/app-context";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Play, Loader2, Terminal } from "lucide-react";
import type { ParseResult } from "@/types";

const PLACEHOLDER = `GET /api/v1/users?page=1&limit=20 HTTP/1.1
Host: api.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
Content-Type: application/json
Accept: application/json`;

export function IdleView() {
  const { state, dispatch } = useApp();

  const handleParse = async () => {
    if (!state.rawText.trim()) return;
    dispatch({ type: "PARSE_START" });
    const start = performance.now();
    try {
      const result = await invoke<ParseResult>("parse_text", { rawText: state.rawText });
      const time = Math.round(performance.now() - start);
      dispatch({ type: "PARSE_SUCCESS", payload: result, time });
    } catch (e) {
      dispatch({ type: "PARSE_ERROR", payload: String(e) });
    }
  };

  const handlePaste = async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (!text.trim()) return;
      dispatch({ type: "SET_RAW_TEXT", payload: text });
      // Auto-parse after paste
      dispatch({ type: "PARSE_START" });
      const start = performance.now();
      const result = await invoke<ParseResult>("parse_text", { rawText: text });
      const time = Math.round(performance.now() - start);
      dispatch({ type: "PARSE_SUCCESS", payload: result, time });
    } catch {
      // clipboard read failed
    }
  };

  const lineCount = state.rawText ? state.rawText.split("\n").length : 0;
  const hasText = state.rawText.trim().length > 0;
  const isParsing = state.parseState === "parsing";

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
            onClick={handleParse}
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
    </div>
  );
}
