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
