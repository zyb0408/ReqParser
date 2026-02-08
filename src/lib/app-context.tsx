import { createContext, useContext, useReducer, type ReactNode } from "react";
import type { ParseNode, ParseResult } from "@/types";

export interface AppState {
  rawText: string;
  parseResult: ParseResult | null;
  parseError: string | null;
  parseState: "idle" | "parsing" | "done" | "error";
  parseTime: number | null;
  selectedNode: ParseNode | null;
  selectedPath: string[];
  clipboardWatching: boolean;
  alwaysOnTop: boolean;
  privacyMask: boolean;
  detailPanelOpen: boolean;
}

export type AppAction =
  | { type: "SET_RAW_TEXT"; payload: string }
  | { type: "PARSE_START" }
  | { type: "PARSE_SUCCESS"; payload: ParseResult; time: number }
  | { type: "PARSE_ERROR"; payload: string }
  | { type: "SELECT_NODE"; payload: { node: ParseNode; path: string[] } }
  | { type: "CLEAR_SELECTION" }
  | { type: "SET_CLIPBOARD_WATCHING"; payload: boolean }
  | { type: "SET_ALWAYS_ON_TOP"; payload: boolean }
  | { type: "TOGGLE_PRIVACY_MASK" }
  | { type: "TOGGLE_DETAIL_PANEL" }
  | { type: "CLEAR_ERROR" }
  | { type: "CLEAR_ALL" };

const initialState: AppState = {
  rawText: "",
  parseResult: null,
  parseError: null,
  parseState: "idle",
  parseTime: null,
  selectedNode: null,
  selectedPath: [],
  clipboardWatching: false,
  alwaysOnTop: false,
  privacyMask: false,
  detailPanelOpen: false,
};

function reducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case "SET_RAW_TEXT":
      return { ...state, rawText: action.payload };
    case "PARSE_START":
      return { ...state, parseState: "parsing", parseError: null };
    case "PARSE_SUCCESS":
      return {
        ...state,
        parseState: "done",
        parseResult: action.payload,
        parseError: null,
        parseTime: action.time,
      };
    case "PARSE_ERROR":
      return {
        ...state,
        parseState: "error",
        parseError: action.payload,
        parseResult: null,
        parseTime: null,
      };
    case "SELECT_NODE":
      return {
        ...state,
        selectedNode: action.payload.node,
        selectedPath: action.payload.path,
        detailPanelOpen: true,
      };
    case "CLEAR_SELECTION":
      return {
        ...state,
        selectedNode: null,
        selectedPath: [],
        detailPanelOpen: false,
      };
    case "SET_CLIPBOARD_WATCHING":
      return { ...state, clipboardWatching: action.payload };
    case "SET_ALWAYS_ON_TOP":
      return { ...state, alwaysOnTop: action.payload };
    case "TOGGLE_PRIVACY_MASK":
      return { ...state, privacyMask: !state.privacyMask };
    case "TOGGLE_DETAIL_PANEL":
      return { ...state, detailPanelOpen: !state.detailPanelOpen };
    case "CLEAR_ERROR":
      return { ...state, parseError: null };
    case "CLEAR_ALL":
      return {
        ...initialState,
        clipboardWatching: state.clipboardWatching,
        alwaysOnTop: state.alwaysOnTop,
        privacyMask: state.privacyMask,
      };
    default:
      return state;
  }
}

const AppContext = createContext<{
  state: AppState;
  dispatch: React.Dispatch<AppAction>;
} | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(reducer, initialState);
  return (
    <AppContext.Provider value={{ state, dispatch }}>
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error("useApp must be used within AppProvider");
  return ctx;
}
