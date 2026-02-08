import { useApp } from "@/lib/app-context";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ScrollArea } from "@/components/ui/scroll-area";
import { KVTreeTable } from "./KVTreeTable";
import { BodyViewer } from "./BodyViewer";
import type { ParseNode } from "@/types";

export function ResultPanel() {
  const { state, dispatch } = useApp();
  const { parseResult, selectedNode, privacyMask } = state;

  if (!parseResult) return null;

  const hasQueryParams = !!parseResult.queryParams?.length;
  const hasBody = !!parseResult.body;

  const handleSelectNode = (node: ParseNode, path: string[]) => {
    dispatch({ type: "SELECT_NODE", payload: { node, path } });
  };

  const headerCount = parseResult.headers.length;
  const queryCount = parseResult.queryParams?.length ?? 0;

  return (
    <div className="flex flex-col h-full animate-fade-up">
      {hasQueryParams || hasBody ? (
        <Tabs defaultValue="headers" className="flex flex-col flex-1 min-h-0">
          <div className="border-b border-border/50">
            <TabsList className="h-9 bg-transparent px-3 gap-4">
              <TabsTrigger
                value="headers"
                className="text-xs bg-transparent px-0 pb-2 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none"
              >
                Headers ({headerCount})
              </TabsTrigger>
              {hasQueryParams && (
                <TabsTrigger
                  value="queryParams"
                  className="text-xs bg-transparent px-0 pb-2 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none"
                >
                  Query Params ({queryCount})
                </TabsTrigger>
              )}
              {hasBody && (
                <TabsTrigger
                  value="body"
                  className="text-xs bg-transparent px-0 pb-2 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none"
                >
                  Body
                </TabsTrigger>
              )}
            </TabsList>
          </div>

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
