import { useState } from "react";
import { useApp } from "@/lib/app-context";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Code2, X, ChevronUp, Play, Loader2 } from "lucide-react";
import type { ParseResult } from "@/types";

export function RequestSummaryStrip() {
  const { state, dispatch } = useApp();
  const [showRaw, setShowRaw] = useState(false);
  const { parseResult, parseTime, rawText } = state;

  if (!parseResult) return null;

  const handleClear = () => {
    dispatch({ type: "CLEAR_ALL" });
  };

  const handleReparse = async () => {
    if (!rawText.trim()) return;
    dispatch({ type: "PARSE_START" });
    const start = performance.now();
    try {
      const result = await invoke<ParseResult>("parse_text", { rawText });
      const time = Math.round(performance.now() - start);
      dispatch({ type: "PARSE_SUCCESS", payload: result, time });
    } catch (e) {
      dispatch({ type: "PARSE_ERROR", payload: String(e) });
    }
  };

  const displayUrl = parseResult.url
    ? parseResult.url.length > 80
      ? parseResult.url.slice(0, 80) + "…"
      : parseResult.url
    : "";

  return (
    <div className="border-b border-border">
      {/* Summary bar */}
      <div className="flex items-center h-10 px-3 gap-2 bg-card/50">
        {parseResult.method && (
          <Badge
            variant="secondary"
            className="shrink-0 font-mono text-xs font-semibold bg-primary/10 text-primary border-none"
          >
            {parseResult.method}
          </Badge>
        )}

        <Tooltip>
          <TooltipTrigger asChild>
            <span className="text-xs font-mono text-muted-foreground truncate min-w-0">
              {displayUrl}
            </span>
          </TooltipTrigger>
          {parseResult.url && parseResult.url.length > 80 && (
            <TooltipContent side="bottom" className="max-w-lg break-all font-mono text-xs">
              {parseResult.url}
            </TooltipContent>
          )}
        </Tooltip>

        {parseTime !== null && (
          <span className="text-[11px] text-[var(--text-dimmed)] shrink-0 ml-auto">
            {parseTime}ms
          </span>
        )}

        <Button
          variant="ghost"
          size="icon"
          className="h-6 w-6 shrink-0 no-drag"
          onClick={() => setShowRaw(!showRaw)}
        >
          {showRaw ? <ChevronUp className="h-3.5 w-3.5" /> : <Code2 className="h-3.5 w-3.5" />}
        </Button>

        <Button
          variant="ghost"
          size="icon"
          className="h-6 w-6 shrink-0 text-muted-foreground hover:text-destructive no-drag"
          onClick={handleClear}
        >
          <X className="h-3.5 w-3.5" />
        </Button>
      </div>

      {/* Expandable raw text */}
      {showRaw && (
        <div className="border-t border-border/50 animate-fade-up">
          <textarea
            className="w-full max-h-64 p-3 font-mono text-xs leading-relaxed bg-muted/20 border-none resize-none focus:outline-none"
            value={rawText}
            onChange={(e) => dispatch({ type: "SET_RAW_TEXT", payload: e.target.value })}
            rows={Math.min(rawText.split("\n").length, 12)}
            spellCheck={false}
          />
          <div className="flex items-center justify-end px-3 py-1.5 border-t border-border/30">
            <Button
              size="sm"
              variant="ghost"
              className="h-6 px-2 text-xs"
              onClick={handleReparse}
              disabled={state.parseState === "parsing" || !rawText.trim()}
            >
              {state.parseState === "parsing" ? (
                <Loader2 className="h-3 w-3 animate-spin mr-1" />
              ) : (
                <Play className="h-3 w-3 mr-1" />
              )}
              重新解析
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
