import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface ParseNode {
  key: string;
  value: string;
  children?: ParseNode[];
  description?: string;
}

interface ParseResult {
  contentType: string;
  method?: string;
  url?: string;
  statusCode?: number;
  statusText?: string;
  protocol?: string;
  headers: ParseNode[];
  queryParams?: ParseNode[];
  body?: string;
  rawText: string;
}

function App() {
  const [inputText, setInputText] = useState("");
  const [parseResult, setParseResult] = useState<ParseResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [watcherEnabled, setWatcherEnabled] = useState(false);
  const [clipboardEvent, setClipboardEvent] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<string>("clipboard-http-detected", (event) => {
      setClipboardEvent(event.payload);
      handleParse(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function handleParse(text?: string) {
    const raw = text ?? inputText;
    try {
      setError(null);
      const result = await invoke<ParseResult>("parse_text", {
        rawText: raw,
      });
      setParseResult(result);
    } catch (e) {
      setError(String(e));
      setParseResult(null);
    }
  }

  async function handleToggleWatcher() {
    try {
      const newState = await invoke<boolean>("toggle_clipboard_watcher", {});
      setWatcherEnabled(newState);
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <main className="p-6 max-w-4xl mx-auto">
      <h1 className="text-2xl font-bold mb-4">ReqParser - Phase 1 Test</h1>

      {/* Clipboard Watcher */}
      <div className="mb-4 flex items-center gap-3">
        <button
          onClick={handleToggleWatcher}
          className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
            watcherEnabled
              ? "bg-green-600 text-white hover:bg-green-700"
              : "bg-secondary text-secondary-foreground hover:bg-secondary/80"
          }`}
        >
          Clipboard Watcher: {watcherEnabled ? "ON" : "OFF"}
        </button>
        {clipboardEvent && (
          <span className="text-sm text-muted-foreground truncate max-w-md">
            Detected: {clipboardEvent.substring(0, 60)}...
          </span>
        )}
      </div>

      {/* Input */}
      <textarea
        className="w-full h-48 border border-input bg-background p-3 font-mono text-sm rounded-md resize-y focus:outline-none focus:ring-2 focus:ring-ring"
        value={inputText}
        onChange={(e) => setInputText(e.target.value)}
        placeholder="Paste HTTP request/response text here..."
      />
      <button
        onClick={() => handleParse()}
        className="mt-2 px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm font-medium hover:bg-primary/90 transition-colors"
      >
        Parse
      </button>

      {/* Error */}
      {error && (
        <p className="mt-3 text-sm text-destructive">Error: {error}</p>
      )}

      {/* Results */}
      {parseResult && (
        <div className="mt-4 space-y-3">
          <div className="flex gap-4 text-sm">
            <span className="font-semibold">
              Type: {parseResult.contentType}
            </span>
            {parseResult.method && <span>Method: {parseResult.method}</span>}
            {parseResult.statusCode && (
              <span>
                Status: {parseResult.statusCode} {parseResult.statusText}
              </span>
            )}
            {parseResult.protocol && (
              <span>Protocol: {parseResult.protocol}</span>
            )}
          </div>

          {parseResult.url && (
            <div className="text-sm">
              <span className="font-semibold">URL: </span>
              <code className="text-xs bg-muted px-1 py-0.5 rounded">
                {parseResult.url}
              </code>
            </div>
          )}

          <div>
            <h3 className="font-semibold text-sm mb-1">
              Headers ({parseResult.headers.length})
            </h3>
            <pre className="bg-muted p-3 rounded-md text-xs overflow-auto max-h-64">
              {JSON.stringify(parseResult.headers, null, 2)}
            </pre>
          </div>

          {parseResult.queryParams && (
            <div>
              <h3 className="font-semibold text-sm mb-1">Query Params</h3>
              <pre className="bg-muted p-3 rounded-md text-xs overflow-auto max-h-40">
                {JSON.stringify(parseResult.queryParams, null, 2)}
              </pre>
            </div>
          )}

          {parseResult.body && (
            <div>
              <h3 className="font-semibold text-sm mb-1">Body</h3>
              <pre className="bg-muted p-3 rounded-md text-xs overflow-auto max-h-40">
                {parseResult.body}
              </pre>
            </div>
          )}
        </div>
      )}
    </main>
  );
}

export default App;
