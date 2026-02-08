import { useApp } from "@/lib/app-context";
import { getHeaderInfo, getSecurityAdvisoriesForHeader } from "@/lib/dictionary";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  X,
  ChevronRight,
  AlertTriangle,
  ExternalLink,
  MousePointer,
  Info,
  Shield,
  Code2,
} from "lucide-react";

const CATEGORY_BADGE_VARIANT: Record<string, "default" | "secondary" | "outline" | "destructive"> = {
  general: "secondary",
  通用: "secondary",
  request: "default",
  请求: "default",
  response: "outline",
  响应: "outline",
  security: "destructive",
  安全: "destructive",
  缓存: "secondary",
  条件请求: "outline",
  内容协商: "secondary",
  CORS: "destructive",
};

export function DetailPanel() {
  const { state, dispatch } = useApp();
  const { selectedNode, selectedPath } = state;

  if (!selectedNode) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-muted-foreground py-16">
        <MousePointer className="h-10 w-10 mb-4 opacity-30" />
        <p className="text-sm">点击左侧表格中的任意行</p>
        <p className="text-xs mt-1">查看字段详细信息</p>
      </div>
    );
  }

  const headerInfo = getHeaderInfo(selectedNode.key);
  const advisories = getSecurityAdvisoriesForHeader(selectedNode.key);

  const handleClose = () => {
    dispatch({ type: "CLEAR_SELECTION" });
  };

  const handleOpenMdn = async (url: string) => {
    try {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl(url);
    } catch {
      window.open(url, "_blank");
    }
  };

  // Format decoded value for display
  let decodedDisplay: string | null = null;
  if (selectedNode.decodedValue) {
    try {
      const parsed = JSON.parse(selectedNode.decodedValue);
      decodedDisplay = JSON.stringify(parsed, null, 2);
    } catch {
      decodedDisplay = selectedNode.decodedValue;
    }
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-start justify-between p-3 border-b border-border">
        <div className="min-w-0">
          <h3 className="text-lg font-semibold truncate">{selectedNode.key}</h3>
          <div className="flex items-center gap-2 mt-1">
            {headerInfo?.category && (
              <Badge variant={CATEGORY_BADGE_VARIANT[headerInfo.category] ?? "secondary"}>
                {headerInfo.category}
              </Badge>
            )}
            {selectedNode.valueType && selectedNode.valueType !== "plain" && (
              <Badge variant="outline" className="text-xs font-mono">
                {selectedNode.valueType}
              </Badge>
            )}
          </div>
          {selectedPath.length > 1 && (
            <p className="text-xs text-muted-foreground mt-1 font-mono">
              {selectedPath.join(" > ")}
            </p>
          )}
        </div>
        <Button variant="ghost" size="icon" className="h-6 w-6 shrink-0" onClick={handleClose}>
          <X className="h-4 w-4" />
        </Button>
      </div>

      <ScrollArea className="flex-1">
        <div className="p-3 space-y-4">
          {/* Value */}
          <DetailSection title="当前值" icon={<Code2 className="h-4 w-4" />} defaultOpen>
            <div className="bg-muted rounded-md p-3 font-mono text-xs break-all overflow-auto max-h-32">
              {selectedNode.value}
            </div>
          </DetailSection>

          {/* Description from dictionary */}
          {headerInfo && (
            <DetailSection title="描述" icon={<Info className="h-4 w-4" />} defaultOpen>
              <p className="text-sm text-muted-foreground leading-relaxed">
                {headerInfo.desc}
              </p>
              {headerInfo.mdn && (
                <button
                  className="text-xs text-primary underline mt-2 flex items-center gap-1 hover:opacity-80"
                  onClick={() => handleOpenMdn(headerInfo.mdn)}
                >
                  <ExternalLink className="h-3 w-3" />
                  MDN 文档
                </button>
              )}
            </DetailSection>
          )}

          {/* Decoded content */}
          {decodedDisplay && (
            <DetailSection title="解码内容" icon={<Code2 className="h-4 w-4" />} defaultOpen>
              <pre className="bg-muted rounded-md p-3 font-mono text-xs overflow-auto max-h-64 whitespace-pre-wrap">
                {decodedDisplay}
              </pre>
            </DetailSection>
          )}

          {/* Value descriptions from dictionary */}
          {headerInfo?.values && Object.keys(headerInfo.values).length > 0 && (
            <DetailSection title="常见值" icon={<Info className="h-4 w-4" />}>
              <div className="space-y-2">
                {Object.entries(headerInfo.values).map(([val, desc]) => (
                  <div key={val}>
                    <code className="text-xs font-mono font-medium bg-muted px-1.5 py-0.5 rounded">
                      {val}
                    </code>
                    <p className="text-xs text-muted-foreground mt-0.5">{desc}</p>
                  </div>
                ))}
              </div>
            </DetailSection>
          )}

          {/* Security advisories */}
          {advisories.length > 0 && (
            <DetailSection title="安全建议" icon={<Shield className="h-4 w-4" />} defaultOpen>
              <div className="space-y-2">
                {advisories.map((advisory) => (
                  <div
                    key={advisory.id}
                    className="bg-destructive/5 border border-destructive/20 rounded-md p-3"
                  >
                    <div className="flex items-start gap-2">
                      <AlertTriangle className="h-4 w-4 text-destructive shrink-0 mt-0.5" />
                      <div className="min-w-0">
                        <p className="text-sm font-medium text-destructive">
                          {advisory.description}
                        </p>
                        <p className="text-xs text-muted-foreground mt-1">
                          {advisory.recommendation}
                        </p>
                        <Badge
                          variant={
                            advisory.risk === "high"
                              ? "destructive"
                              : advisory.risk === "medium"
                                ? "secondary"
                                : "outline"
                          }
                          className="mt-1 text-[10px]"
                        >
                          {advisory.risk}
                        </Badge>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </DetailSection>
          )}
        </div>
      </ScrollArea>
    </div>
  );
}

interface DetailSectionProps {
  title: string;
  icon?: React.ReactNode;
  children: React.ReactNode;
  defaultOpen?: boolean;
}

function DetailSection({ title, icon, children, defaultOpen = false }: DetailSectionProps) {
  return (
    <Collapsible defaultOpen={defaultOpen}>
      <CollapsibleTrigger className="flex items-center gap-2 w-full text-sm font-medium hover:text-foreground text-muted-foreground py-1 group">
        <ChevronRight className="h-3.5 w-3.5 transition-transform duration-150 group-data-[state=open]:rotate-90" />
        {icon}
        {title}
      </CollapsibleTrigger>
      <CollapsibleContent className="mt-2">
        {children}
      </CollapsibleContent>
    </Collapsible>
  );
}
