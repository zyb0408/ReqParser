import { useApp } from "@/lib/app-context";
import { invoke } from "@tauri-apps/api/core";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Play, Loader2 } from "lucide-react";
import type { ParseResult } from "@/types";

export function InputPanel() {
  const { state, dispatch } = useApp();

  const lineCount = state.rawText ? state.rawText.split("\n").length : 0;
  const charCount = state.rawText.length;

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

  const contentTypeLabel = state.parseResult
    ? {
        request: "Request",
        response: "Response",
        headersOnly: "Headers Only",
        unknown: "Unknown",
      }[state.parseResult.contentType]
    : null;

  return (
    <div className="flex flex-col h-full">
      {/* Header with parse button */}
      <div className="flex items-center justify-between h-9 px-3 border-b border-border bg-muted/30">
        <span className="text-sm font-medium">Input</span>
        <Button
          size="sm"
          variant="ghost"
          className="h-6 px-2 text-xs"
          onClick={handleParse}
          disabled={state.parseState === "parsing" || !state.rawText.trim()}
        >
          {state.parseState === "parsing" ? (
            <Loader2 className="h-3 w-3 animate-spin" />
          ) : (
            <Play className="h-3 w-3" />
          )}
        </Button>
      </div>

      {/* Textarea */}
      <ScrollArea className="flex-1">
        <textarea
          className="w-full h-full min-h-[200px] p-3 font-mono text-sm leading-relaxed bg-muted/30 border-none resize-none focus:outline-none"
          value={state.rawText}
          onChange={(e) => dispatch({ type: "SET_RAW_TEXT", payload: e.target.value })}
          placeholder="粘贴 HTTP 请求/响应文本..."
          spellCheck={false}
        />
      </ScrollArea>

      {/* Request summary */}
      {state.parseResult && state.parseResult.method && (
        <div className="px-3 py-1.5 border-t border-border bg-muted/20">
          <div className="flex items-center gap-2 overflow-hidden">
            <Badge variant="secondary" className="shrink-0 font-mono text-xs">
              {state.parseResult.method}
            </Badge>
            <span className="text-xs font-mono text-muted-foreground truncate">
              {state.parseResult.url}
            </span>
          </div>
        </div>
      )}

      {/* Meta info */}
      <div className="flex items-center justify-between h-7 px-3 text-xs text-muted-foreground border-t border-border bg-muted/10">
        <div className="flex items-center gap-3">
          {contentTypeLabel && (
            <Badge variant="outline" className="h-4 text-[10px] px-1.5 py-0">
              {contentTypeLabel}
            </Badge>
          )}
        </div>
        <div className="flex items-center gap-3">
          <span>L:{lineCount}</span>
          <span>C:{charCount.toLocaleString()}</span>
        </div>
      </div>
    </div>
  );
}
