import { ScrollArea } from "@/components/ui/scroll-area";

interface BodyViewerProps {
  body: string;
}

type TokenType = "key" | "string" | "number" | "boolean" | "null" | "bracket" | "text";

interface Token {
  type: TokenType;
  text: string;
}

function tokenizeLine(line: string): Token[] {
  const tokens: Token[] = [];
  let remaining = line;

  while (remaining.length > 0) {
    // Match key: "..."  :
    const keyMatch = remaining.match(/^(\s*)("(?:[^"\\]|\\.)*")(\s*:)/);
    if (keyMatch) {
      if (keyMatch[1]) tokens.push({ type: "text", text: keyMatch[1] });
      tokens.push({ type: "key", text: keyMatch[2] });
      tokens.push({ type: "text", text: keyMatch[3] });
      remaining = remaining.slice(keyMatch[0].length);
      continue;
    }

    // Match string value: "..."
    const strMatch = remaining.match(/^(\s*)("(?:[^"\\]|\\.)*")/);
    if (strMatch) {
      if (strMatch[1]) tokens.push({ type: "text", text: strMatch[1] });
      tokens.push({ type: "string", text: strMatch[2] });
      remaining = remaining.slice(strMatch[0].length);
      continue;
    }

    // Match number
    const numMatch = remaining.match(/^(\s*)(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/);
    if (numMatch) {
      if (numMatch[1]) tokens.push({ type: "text", text: numMatch[1] });
      tokens.push({ type: "number", text: numMatch[2] });
      remaining = remaining.slice(numMatch[0].length);
      continue;
    }

    // Match boolean
    const boolMatch = remaining.match(/^(\s*)(true|false)/);
    if (boolMatch) {
      if (boolMatch[1]) tokens.push({ type: "text", text: boolMatch[1] });
      tokens.push({ type: "boolean", text: boolMatch[2] });
      remaining = remaining.slice(boolMatch[0].length);
      continue;
    }

    // Match null
    const nullMatch = remaining.match(/^(\s*)(null)/);
    if (nullMatch) {
      if (nullMatch[1]) tokens.push({ type: "text", text: nullMatch[1] });
      tokens.push({ type: "null", text: nullMatch[2] });
      remaining = remaining.slice(nullMatch[0].length);
      continue;
    }

    // Match brackets
    const bracketMatch = remaining.match(/^([{}[\]])/);
    if (bracketMatch) {
      tokens.push({ type: "bracket", text: bracketMatch[1] });
      remaining = remaining.slice(1);
      continue;
    }

    // Consume one character as plain text
    tokens.push({ type: "text", text: remaining[0] });
    remaining = remaining.slice(1);
  }

  return tokens;
}

const tokenClassMap: Record<TokenType, string> = {
  key: "syntax-key",
  string: "syntax-string",
  number: "syntax-number",
  boolean: "syntax-boolean",
  null: "syntax-null",
  bracket: "syntax-bracket",
  text: "",
};

function highlightJson(jsonStr: string): React.ReactNode[] {
  const lines = jsonStr.split("\n");
  return lines.map((line, i) => {
    const tokens = tokenizeLine(line);
    return (
      <div key={i} className="leading-relaxed">
        {tokens.map((token, j) => {
          const cls = tokenClassMap[token.type];
          return cls ? (
            <span key={j} className={cls}>{token.text}</span>
          ) : (
            <span key={j}>{token.text}</span>
          );
        })}
      </div>
    );
  });
}

export function BodyViewer({ body }: BodyViewerProps) {
  let formatted = body;
  let isJson = false;

  try {
    const parsed = JSON.parse(body);
    formatted = JSON.stringify(parsed, null, 2);
    isJson = true;
  } catch {
    // not JSON, show as-is
  }

  return (
    <ScrollArea className="h-full">
      <pre className="p-3 font-mono text-xs overflow-auto">
        {isJson ? (
          <code>{highlightJson(formatted)}</code>
        ) : (
          <code>{formatted}</code>
        )}
      </pre>
    </ScrollArea>
  );
}
