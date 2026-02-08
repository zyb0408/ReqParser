import { useState, useCallback } from "react";
import { cn } from "@/lib/utils";
import { TypeBadge } from "./TypeBadge";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Button } from "@/components/ui/button";
import {
  ChevronRight,
  Copy,
  Check,
  ArrowRightLeft,
} from "lucide-react";
import type { ParseNode } from "@/types";

const SENSITIVE_KEYS = /token|auth|key|secret|password|cookie/i;

interface KVTreeTableProps {
  nodes: ParseNode[];
  title: string;
  selectedNode: ParseNode | null;
  onSelectNode: (node: ParseNode, path: string[]) => void;
  privacyMask: boolean;
}

export function KVTreeTable({
  nodes,
  title,
  selectedNode,
  onSelectNode,
  privacyMask,
}: KVTreeTableProps) {
  const [collapsedAll, setCollapsedAll] = useState(false);
  const [sectionCollapsed, setSectionCollapsed] = useState(false);

  return (
    <div className="border-b border-border/50">
      {/* Section header */}
      <button
        className="flex items-center justify-between w-full h-9 px-3 bg-muted/30 text-sm font-medium hover:bg-muted/50 transition-colors"
        onClick={() => setSectionCollapsed(!sectionCollapsed)}
      >
        <div className="flex items-center gap-2">
          <ChevronRight
            className={cn(
              "h-4 w-4 transition-transform duration-150",
              !sectionCollapsed && "rotate-90"
            )}
          />
          <span>{title}</span>
          <span className="text-xs text-muted-foreground">({nodes.length})</span>
        </div>
        {!sectionCollapsed && nodes.some((n) => n.children?.length) && (
          <span
            className="text-xs text-muted-foreground hover:text-foreground cursor-pointer"
            onClick={(e) => {
              e.stopPropagation();
              setCollapsedAll(!collapsedAll);
            }}
          >
            {collapsedAll ? "展开全部" : "折叠全部"}
          </span>
        )}
      </button>

      {/* Rows */}
      {!sectionCollapsed && (
        <div>
          {nodes.map((node, i) => (
            <TreeRow
              key={`${node.key}-${i}`}
              node={node}
              depth={0}
              selectedNode={selectedNode}
              onSelectNode={onSelectNode}
              privacyMask={privacyMask}
              parentPath={[title]}
              forceCollapsed={collapsedAll}
            />
          ))}
        </div>
      )}
    </div>
  );
}

interface TreeRowProps {
  node: ParseNode;
  depth: number;
  selectedNode: ParseNode | null;
  onSelectNode: (node: ParseNode, path: string[]) => void;
  privacyMask: boolean;
  parentPath: string[];
  forceCollapsed?: boolean;
  isLast?: boolean;
}

function TreeRow({
  node,
  depth,
  selectedNode,
  onSelectNode,
  privacyMask,
  parentPath,
  forceCollapsed,
  isLast,
}: TreeRowProps) {
  const [expanded, setExpanded] = useState(true);
  const [showDecoded, setShowDecoded] = useState(true);
  const [copiedKey, setCopiedKey] = useState(false);
  const [copiedValue, setCopiedValue] = useState(false);

  const hasChildren = !!node.children?.length;
  const isSelected = selectedNode === node;
  const isSensitive = SENSITIVE_KEYS.test(node.key);
  const isExpanded = forceCollapsed ? false : expanded;
  const currentPath = [...parentPath, node.key];

  const displayValue =
    privacyMask && isSensitive
      ? "***MASKED***"
      : showDecoded && node.decodedValue
        ? node.decodedValue
        : node.value;

  const copyToClipboard = useCallback(async (text: string, type: "key" | "value") => {
    try {
      await navigator.clipboard.writeText(text);
      if (type === "key") {
        setCopiedKey(true);
        setTimeout(() => setCopiedKey(false), 1500);
      } else {
        setCopiedValue(true);
        setTimeout(() => setCopiedValue(false), 1500);
      }
    } catch {
      // copy failed
    }
  }, []);

  return (
    <>
      <div
        className={cn(
          "group flex items-center h-9 px-3 border-b border-border/50 transition-colors duration-75 cursor-pointer",
          isSelected
            ? "bg-accent border-l-2 border-l-primary"
            : "hover:bg-accent/50"
        )}
        onClick={() => onSelectNode(node, currentPath)}
      >
        {/* Indent + tree line */}
        <div className="flex items-center shrink-0" style={{ width: depth * 24 }}>
          {depth > 0 && (
            <div className="relative h-full" style={{ width: depth * 24 }}>
              {/* Vertical tree line */}
              <div
                className="absolute border-l border-[var(--tree-line)]"
                style={{
                  left: (depth - 1) * 24 + 12,
                  top: 0,
                  height: isLast ? "50%" : "100%",
                }}
              />
              {/* Horizontal connector */}
              <div
                className="absolute border-b border-[var(--tree-line)]"
                style={{
                  left: (depth - 1) * 24 + 12,
                  top: "50%",
                  width: 12,
                }}
              />
            </div>
          )}
        </div>

        {/* Expand toggle */}
        <div className="w-5 shrink-0 flex items-center justify-center">
          {hasChildren ? (
            <button
              className="p-0.5 hover:bg-muted rounded-sm"
              onClick={(e) => {
                e.stopPropagation();
                setExpanded(!expanded);
              }}
            >
              <ChevronRight
                className={cn(
                  "h-3.5 w-3.5 transition-transform duration-150",
                  isExpanded && "rotate-90"
                )}
              />
            </button>
          ) : null}
        </div>

        {/* Key */}
        <div
          className="w-[160px] min-w-[100px] shrink-0 font-mono text-[13px] font-medium truncate mr-2 cursor-pointer hover:text-primary"
          onClick={(e) => {
            e.stopPropagation();
            copyToClipboard(node.key, "key");
          }}
        >
          {node.key}
        </div>

        {/* Value */}
        <div className="flex-1 min-w-0 flex items-center gap-2">
          <Tooltip>
            <TooltipTrigger asChild>
              <span
                className={cn(
                  "font-mono text-[13px] truncate",
                  privacyMask && isSensitive && "text-muted-foreground italic"
                )}
              >
                {displayValue}
              </span>
            </TooltipTrigger>
            <TooltipContent
              side="bottom"
              className="max-w-md break-all font-mono text-xs"
            >
              {node.value}
            </TooltipContent>
          </Tooltip>

          {showDecoded && node.decodedValue && !(privacyMask && isSensitive) && (
            <span className="shrink-0 inline-block w-1.5 h-1.5 rounded-full bg-primary/50" />
          )}
        </div>

        {/* Type badge */}
        <div className="w-16 shrink-0 flex items-center justify-center">
          <TypeBadge type={node.valueType} />
        </div>

        {/* Actions */}
        <div className="w-20 shrink-0 flex items-center justify-end gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity duration-100">
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6"
            onClick={(e) => {
              e.stopPropagation();
              copyToClipboard(node.key, "key");
            }}
          >
            {copiedKey ? (
              <Check className="h-3 w-3 text-green-500" />
            ) : (
              <Copy className="h-3 w-3" />
            )}
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6"
            onClick={(e) => {
              e.stopPropagation();
              copyToClipboard(node.value, "value");
            }}
          >
            {copiedValue ? (
              <Check className="h-3 w-3 text-green-500" />
            ) : (
              <Copy className="h-3 w-3" />
            )}
          </Button>
          {node.decodedValue && (
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6"
              onClick={(e) => {
                e.stopPropagation();
                setShowDecoded(!showDecoded);
              }}
            >
              <ArrowRightLeft className="h-3 w-3" />
            </Button>
          )}
        </div>
      </div>

      {/* Children */}
      {hasChildren && isExpanded && (
        <div>
          {node.children!.map((child, i) => (
            <TreeRow
              key={`${child.key}-${i}`}
              node={child}
              depth={depth + 1}
              selectedNode={selectedNode}
              onSelectNode={onSelectNode}
              privacyMask={privacyMask}
              parentPath={currentPath}
              forceCollapsed={forceCollapsed}
              isLast={i === node.children!.length - 1}
            />
          ))}
        </div>
      )}
    </>
  );
}
