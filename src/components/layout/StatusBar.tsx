import { useApp } from "@/lib/app-context";

export function StatusBar() {
  const { state } = useApp();

  const itemCount = state.parseResult
    ? state.parseResult.headers.length + (state.parseResult.queryParams?.length ?? 0)
    : 0;

  return (
    <div className="flex items-center h-6 px-4 text-[11px] text-muted-foreground bg-card/50 backdrop-blur-sm border-t border-border gap-4">
      <div className="flex items-center gap-1.5">
        <span
          className={`inline-block w-2 h-2 rounded-full ${state.clipboardWatching ? "animate-subtle-pulse" : ""}`}
          style={{
            backgroundColor: state.clipboardWatching
              ? "var(--status-active)"
              : "var(--status-inactive)",
          }}
        />
        <span>{state.clipboardWatching ? "剪贴板监听中" : "监听已关闭"}</span>
      </div>

      {state.parseTime !== null && (
        <span>解析: {state.parseTime}ms</span>
      )}

      {state.parseResult && (
        <span>共 {itemCount} 个字段</span>
      )}

      <div className="flex-1" />
      <span>v0.1.0</span>
    </div>
  );
}
