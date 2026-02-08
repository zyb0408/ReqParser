import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useApp } from "@/lib/app-context";
import type { HistoryEntrySummary, ParseResult } from "@/types";

export function useParse() {
  const { state, dispatch } = useApp();

  const refreshHistory = useCallback(async () => {
    try {
      const list = await invoke<HistoryEntrySummary[]>("history_list");
      dispatch({ type: "SET_HISTORY_LIST", payload: list });
    } catch {
      // history refresh failed silently
    }
  }, [dispatch]);

  const parse = useCallback(
    async (text?: string) => {
      const raw = text ?? state.rawText;
      if (!raw.trim()) return;

      dispatch({ type: "PARSE_START" });
      const start = performance.now();
      try {
        const result = await invoke<ParseResult>("parse_text", { rawText: raw });
        const time = Math.round(performance.now() - start);
        dispatch({ type: "PARSE_SUCCESS", payload: result, time });

        // fire-and-forget: save to history and refresh list
        invoke("history_save", { rawText: raw, parseResult: result })
          .then(() => refreshHistory())
          .catch(() => {});
      } catch (e) {
        dispatch({ type: "PARSE_ERROR", payload: String(e) });
      }
    },
    [state.rawText, dispatch, refreshHistory],
  );

  return { parse, refreshHistory };
}
