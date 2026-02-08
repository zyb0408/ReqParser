import { ClipboardPaste } from "lucide-react";

export function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center h-full text-muted-foreground py-16">
      <ClipboardPaste className="h-12 w-12 mb-4 opacity-30" />
      <p className="text-sm font-medium mb-1">粘贴 HTTP 请求/响应文本开始解析</p>
      <p className="text-xs">支持 Request, Response, Headers, JSON 等格式</p>
      <div className="mt-4 text-xs bg-muted/50 px-3 py-1.5 rounded-md font-mono">
        Cmd+V 快捷粘贴
      </div>
    </div>
  );
}
