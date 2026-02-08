import { useTheme } from "@/components/ThemeProvider";
import { useApp } from "@/lib/app-context";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Toggle } from "@/components/ui/toggle";
import { Separator } from "@/components/ui/separator";
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
    if (theme === "light") setTheme("dark");
    else if (theme === "dark") setTheme("system");
    else setTheme("light");
  };

  return (
    <div className="flex items-center h-12 px-4 border-b border-border bg-background">
      {/* Brand */}
      <div className="flex items-center gap-2">
        <Terminal className="h-4 w-4" />
        <span className="text-sm font-semibold">ReqParser</span>
        <span className="text-xs text-muted-foreground hidden min-[1000px]:inline">v0.1.0</span>
      </div>

      <Separator orientation="vertical" className="h-6 mx-3" />

      {/* Actions */}
      <div className="flex items-center gap-1">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="outline" size="sm" onClick={handlePaste}>
              <ClipboardPaste className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>粘贴 (Cmd+V)</TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="outline" size="sm" onClick={handleClear}>
              <Trash2 className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>清空</TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              size="sm"
              onClick={() => handleParse()}
              disabled={state.parseState === "parsing" || !state.rawText.trim()}
            >
              {state.parseState === "parsing" ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Play className="h-4 w-4" />
              )}
              <span className="ml-1">解析</span>
            </Button>
          </TooltipTrigger>
          <TooltipContent>解析 (Cmd+Enter)</TooltipContent>
        </Tooltip>
      </div>

      <div className="flex-1" />

      {/* Toggles */}
      <div className="flex items-center gap-1">
        <Tooltip>
          <TooltipTrigger asChild>
            <Toggle
              size="sm"
              pressed={state.clipboardWatching}
              onPressedChange={handleToggleClipboard}
              aria-label="剪贴板监听"
            >
              {state.clipboardWatching ? (
                <ClipboardCheck className="h-4 w-4" />
              ) : (
                <ClipboardList className="h-4 w-4" />
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
            >
              {state.privacyMask ? (
                <EyeOff className="h-4 w-4" />
              ) : (
                <Eye className="h-4 w-4" />
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
            >
              {resolvedTheme === "dark" ? (
                <Moon className="h-4 w-4" />
              ) : (
                <Sun className="h-4 w-4" />
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
