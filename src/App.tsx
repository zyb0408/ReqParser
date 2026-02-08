import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { TooltipProvider } from "@/components/ui/tooltip";
import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from "@/components/ui/resizable";
import { ThemeProvider } from "@/components/ThemeProvider";
import { AppProvider, useApp } from "@/lib/app-context";
import { Toolbar } from "@/components/toolbar/Toolbar";
import { StatusBar } from "@/components/layout/StatusBar";
import { InputPanel } from "@/components/panels/InputPanel";
import { ResultPanel } from "@/components/kv/ResultPanel";
import { DetailPanel } from "@/components/detail/DetailPanel";
import type { ParseResult } from "@/types";

function AppContent() {
  const { state, dispatch } = useApp();

  // Listen for clipboard HTTP detection events
  useEffect(() => {
    const unlisten = listen<string>("clipboard-http-detected", async (event) => {
      const text = event.payload;
      dispatch({ type: "SET_RAW_TEXT", payload: text });
      dispatch({ type: "PARSE_START" });
      const start = performance.now();
      try {
        const result = await invoke<ParseResult>("parse_text", { rawText: text });
        const time = Math.round(performance.now() - start);
        dispatch({ type: "PARSE_SUCCESS", payload: result, time });
      } catch (e) {
        dispatch({ type: "PARSE_ERROR", payload: String(e) });
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [dispatch]);

  // Keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const isMeta = e.metaKey || e.ctrlKey;

      if (isMeta && e.key === "Enter") {
        e.preventDefault();
        if (state.rawText.trim()) {
          (async () => {
            dispatch({ type: "PARSE_START" });
            const start = performance.now();
            try {
              const result = await invoke<ParseResult>("parse_text", {
                rawText: state.rawText,
              });
              const time = Math.round(performance.now() - start);
              dispatch({ type: "PARSE_SUCCESS", payload: result, time });
            } catch (err) {
              dispatch({ type: "PARSE_ERROR", payload: String(err) });
            }
          })();
        }
      }

      if (e.key === "Escape") {
        dispatch({ type: "CLEAR_SELECTION" });
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [state.rawText, dispatch]);

  // Auto-dismiss error toast
  useEffect(() => {
    if (!state.parseError) return;
    const timer = setTimeout(() => {
      dispatch({ type: "CLEAR_ERROR" });
    }, 5000);
    return () => clearTimeout(timer);
  }, [state.parseError, dispatch]);

  return (
    <div className="flex flex-col h-screen">
      <Toolbar />

      <div className="flex-1 min-h-0">
        <ResizablePanelGroup orientation="horizontal">
          {/* Left: Input Panel */}
          <ResizablePanel defaultSize={25} minSize={15} maxSize={35}>
            <InputPanel />
          </ResizablePanel>

          <ResizableHandle withHandle />

          {/* Middle: KV Tree / Result Panel */}
          <ResizablePanel defaultSize={50} minSize={30}>
            <ResultPanel />
          </ResizablePanel>

          <ResizableHandle withHandle />

          {/* Right: Detail Panel */}
          <ResizablePanel defaultSize={25} minSize={15} maxSize={35}>
            <DetailPanel />
          </ResizablePanel>
        </ResizablePanelGroup>
      </div>

      <StatusBar />

      {/* Error toast */}
      {state.parseError && (
        <div className="fixed bottom-10 left-1/2 -translate-x-1/2 bg-destructive text-destructive-foreground px-4 py-2 rounded-md text-sm shadow-lg animate-in fade-in-0 slide-in-from-bottom-4">
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
