import { useApp } from "@/lib/app-context";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ScrollArea } from "@/components/ui/scroll-area";
import { KVTreeTable } from "./KVTreeTable";
import { BodyViewer } from "./BodyViewer";
import { EmptyState } from "./EmptyState";
import type { ParseNode } from "@/types";

export function ResultPanel() {
  const { state, dispatch } = useApp();
  const { parseResult, selectedNode, privacyMask } = state;

  if (!parseResult) {
    return <EmptyState />;
  }

  const hasQueryParams = !!parseResult.queryParams?.length;
  const hasBody = !!parseResult.body;

  const handleSelectNode = (node: ParseNode, path: string[]) => {
    dispatch({ type: "SELECT_NODE", payload: { node, path } });
  };

  return (
    <div className="flex flex-col h-full">
      {/* Table header */}
      <div className="flex items-center h-8 px-3 bg-muted/50 text-xs font-medium text-muted-foreground uppercase tracking-wider border-b border-border sticky top-0 z-10">
        <div className="w-5 shrink-0" />
        <div className="w-[160px] min-w-[100px] shrink-0">Key</div>
        <div className="flex-1">Value</div>
        <div className="w-16 shrink-0 text-center">Type</div>
        <div className="w-20 shrink-0 text-right">Actions</div>
      </div>

      {hasQueryParams || hasBody ? (
        <Tabs defaultValue="headers" className="flex flex-col flex-1 min-h-0">
          <TabsList className="h-8 mx-3 mt-1 mb-0">
            <TabsTrigger value="headers" className="text-xs">
              Headers
            </TabsTrigger>
            {hasQueryParams && (
              <TabsTrigger value="queryParams" className="text-xs">
                Query Params
              </TabsTrigger>
            )}
            {hasBody && (
              <TabsTrigger value="body" className="text-xs">
                Body
              </TabsTrigger>
            )}
          </TabsList>

          <ScrollArea className="flex-1">
            <TabsContent value="headers" className="mt-0">
              <KVTreeTable
                nodes={parseResult.headers}
                title="Headers"
                selectedNode={selectedNode}
                onSelectNode={handleSelectNode}
                privacyMask={privacyMask}
              />
            </TabsContent>

            {hasQueryParams && (
              <TabsContent value="queryParams" className="mt-0">
                <KVTreeTable
                  nodes={parseResult.queryParams!}
                  title="Query Params"
                  selectedNode={selectedNode}
                  onSelectNode={handleSelectNode}
                  privacyMask={privacyMask}
                />
              </TabsContent>
            )}

            {hasBody && (
              <TabsContent value="body" className="mt-0">
                <BodyViewer body={parseResult.body!} />
              </TabsContent>
            )}
          </ScrollArea>
        </Tabs>
      ) : (
        <ScrollArea className="flex-1">
          <KVTreeTable
            nodes={parseResult.headers}
            title="Headers"
            selectedNode={selectedNode}
            onSelectNode={handleSelectNode}
            privacyMask={privacyMask}
          />
        </ScrollArea>
      )}
    </div>
  );
}
