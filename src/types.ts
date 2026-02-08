export interface ParseNode {
  key: string;
  value: string;
  children?: ParseNode[];
  description?: string;
  decodedValue?: string;
  valueType?: string;
}

export type HttpContentType = "request" | "response" | "headersOnly" | "unknown";

export interface ParseResult {
  contentType: HttpContentType;
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

export interface HistoryEntrySummary {
  id: string;
  title: string;
  method?: string;
  url?: string;
  createdAt: string;
}

export interface HistoryEntry {
  id: string;
  title: string;
  rawText: string;
  parseResult: ParseResult;
  createdAt: string;
}
