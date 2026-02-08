import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { TooltipProvider } from "@/components/ui/tooltip";
import { ThemeProvider } from "@/components/ThemeProvider";
import { AppProvider, useApp } from "@/lib/app-context";
import { useParse } from "@/lib/use-parse";
import { Toolbar } from "@/components/toolbar/Toolbar";
import { StatusBar } from "@/components/layout/StatusBar";
import { IdleView } from "@/components/panels/IdleView";
import { RequestSummaryStrip } from "@/components/panels/RequestSummaryStrip";
import { ResultPanel } from "@/components/kv/ResultPanel";
import { DetailPanel } from "@/components/detail/DetailPanel";
import { HistoryList } from "@/components/history/HistoryList";

function AppContent() {
  const { state, dispatch } = useApp();
  const { parse, refreshHistory } = useParse();

  // Load history on startup
  useEffect(() => {
    refreshHistory();
  }, [refreshHistory]);

  // Listen for clipboard HTTP detection events
  useEffect(() => {
    const unlisten = listen<string>("clipboard-http-detected", (event) => {
      const text = event.payload;
      dispatch({ type: "SET_RAW_TEXT", payload: text });
      parse(text);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [dispatch, parse]);

  // Keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const isMeta = e.metaKey || e.ctrlKey;

      if (isMeta && e.key === "Enter") {
        e.preventDefault();
        if (state.rawText.trim()) {
          parse();
        }
      }

      if (e.key === "Escape") {
        if (state.historyOpen) {
          dispatch({ type: "TOGGLE_HISTORY" });
        } else {
          dispatch({ type: "CLEAR_SELECTION" });
        }
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [state.rawText, state.historyOpen, dispatch, parse]);

  // Auto-dismiss error toast
  useEffect(() => {
    if (!state.parseError) return;
    const timer = setTimeout(() => {
      dispatch({ type: "CLEAR_ERROR" });
    }, 5000);
    return () => clearTimeout(timer);
  }, [state.parseError, dispatch]);

  const hasResult = !!state.parseResult;
  const hasSelection = !!state.selectedNode;

  return (
    <div className="flex flex-col h-screen">
      <Toolbar />

      {!hasResult && state.historyOpen ? (
        <HistoryList />
      ) : !hasResult ? (
        <IdleView />
      ) : (
        <div className="flex flex-col flex-1 min-h-0">
          <RequestSummaryStrip />

          <div className="flex-1 min-h-0 flex">
            <div className={hasSelection ? "w-[65%] h-full min-w-0" : "w-full h-full"}>
              <ResultPanel />
            </div>
            {hasSelection && (
              <div className="w-[35%] h-full border-l border-border animate-slide-in-right overflow-hidden">
                <DetailPanel />
              </div>
            )}
          </div>
        </div>
      )}

      <StatusBar />

      {/* Error toast */}
      {state.parseError && (
        <div className="fixed bottom-10 left-1/2 -translate-x-1/2 bg-destructive text-destructive-foreground px-4 py-2 rounded-lg text-sm shadow-2xl animate-fade-up">
          {state.parseError}
        </div>
      )}
    </div>
  );
}

function App() {
  return (
    <ThemeProvider>
      <TooltipProvider delayDuration={300}>
        <AppProvider>
          <AppContent />
        </AppProvider>
      </TooltipProvider>
    </ThemeProvider>
  );
}

export default App;
