import { useTheme } from "@/components/ThemeProvider";
import { useApp } from "@/lib/app-context";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Toggle } from "@/components/ui/toggle";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  ClipboardPaste,
  Trash2,
  Play,
  Loader2,
  ClipboardCheck,
  ClipboardList,
  Eye,
  EyeOff,
  Sun,
  Moon,
  Terminal,
} from "lucide-react";
import type { ParseResult } from "@/types";

export function Toolbar() {
  const { state, dispatch } = useApp();
  const { theme, setTheme, resolvedTheme } = useTheme();

  const handlePaste = async () => {
    try {
      const text = await navigator.clipboard.readText();
      dispatch({ type: "SET_RAW_TEXT", payload: text });
      handleParse(text);
    } catch {
      // clipboard read failed silently
    }
  };

  const handleClear = () => {
    dispatch({ type: "CLEAR_ALL" });
  };

  const handleParse = async (text?: string) => {
    const raw = text ?? state.rawText;
    if (!raw.trim()) return;
    dispatch({ type: "PARSE_START" });
    const start = performance.now();
    try {
      const result = await invoke<ParseResult>("parse_text", { rawText: raw });
      const time = Math.round(performance.now() - start);
      dispatch({ type: "PARSE_SUCCESS", payload: result, time });
    } catch (e) {
      dispatch({ type: "PARSE_ERROR", payload: String(e) });
    }
  };

  const handleToggleClipboard = async () => {
    try {
      const newState = await invoke<boolean>("toggle_clipboard_watcher", {});
      dispatch({ type: "SET_CLIPBOARD_WATCHING", payload: newState });
    } catch {
      // toggle failed silently
    }
  };

  const handleToggleTheme = () => {
    // dark → light → system → dark
    // Ensures first click from default dark always produces visible change
    if (theme === "dark") setTheme("light");
    else if (theme === "light") setTheme("system");
    else setTheme("dark");
  };

  const hasResult = !!state.parseResult;

  return (
    <div className="flex items-center h-10 px-3 border-b border-border bg-card/80 backdrop-blur-sm drag-region">
      {/* Brand */}
      <div className="flex items-center gap-2 no-drag">
        <div className="flex items-center justify-center w-6 h-6 rounded-md bg-primary/10">
          <Terminal className="h-3.5 w-3.5 text-primary" />
        </div>
        <span className="text-sm font-semibold">ReqParser</span>
      </div>

      <div className="w-3" />

      {/* Actions — only show parse-related buttons when there's a result */}
      <div className="flex items-center gap-0.5 no-drag">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="ghost" size="icon" className="h-7 w-7" onClick={handlePaste}>
              <ClipboardPaste className="h-3.5 w-3.5" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>粘贴 (Cmd+V)</TooltipContent>
        </Tooltip>

        {hasResult && (
          <>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button variant="ghost" size="icon" className="h-7 w-7" onClick={handleClear}>
                  <Trash2 className="h-3.5 w-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>清空</TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-7 w-7"
                  onClick={() => handleParse()}
                  disabled={state.parseState === "parsing" || !state.rawText.trim()}
                >
                  {state.parseState === "parsing" ? (
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  ) : (
                    <Play className="h-3.5 w-3.5" />
                  )}
                </Button>
              </TooltipTrigger>
              <TooltipContent>重新解析 (Cmd+Enter)</TooltipContent>
            </Tooltip>
          </>
        )}
      </div>

      <div className="flex-1" />

      {/* Toggles */}
      <div className="flex items-center gap-0.5 no-drag">
        <Tooltip>
          <TooltipTrigger asChild>
            <Toggle
              size="sm"
              pressed={state.clipboardWatching}
              onPressedChange={handleToggleClipboard}
              aria-label="剪贴板监听"
              className="h-7 w-7 p-0"
            >
              {state.clipboardWatching ? (
                <ClipboardCheck className="h-3.5 w-3.5" />
              ) : (
                <ClipboardList className="h-3.5 w-3.5" />
              )}
            </Toggle>
          </TooltipTrigger>
          <TooltipContent>剪贴板监听</TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger asChild>
            <Toggle
              size="sm"
              pressed={state.privacyMask}
              onPressedChange={() => dispatch({ type: "TOGGLE_PRIVACY_MASK" })}
              aria-label="隐私脱敏"
              className="h-7 w-7 p-0"
            >
              {state.privacyMask ? (
                <EyeOff className="h-3.5 w-3.5" />
              ) : (
                <Eye className="h-3.5 w-3.5" />
              )}
            </Toggle>
          </TooltipTrigger>
          <TooltipContent>隐私脱敏</TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger asChild>
            <Toggle
              size="sm"
              pressed={resolvedTheme === "dark"}
              onPressedChange={handleToggleTheme}
              aria-label="主题切换"
              className="h-7 w-7 p-0"
            >
              {resolvedTheme === "dark" ? (
                <Moon className="h-3.5 w-3.5" />
              ) : (
                <Sun className="h-3.5 w-3.5" />
              )}
            </Toggle>
          </TooltipTrigger>
          <TooltipContent>
            主题: {theme === "system" ? "跟随系统" : theme === "dark" ? "深色" : "浅色"}
          </TooltipContent>
        </Tooltip>
      </div>
    </div>
  );
}
