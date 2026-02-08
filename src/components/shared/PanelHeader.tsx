import { cn } from "@/lib/utils";

interface PanelHeaderProps {
  title: string;
  count?: number;
  children?: React.ReactNode;
  className?: string;
}

export function PanelHeader({ title, count, children, className }: PanelHeaderProps) {
  return (
    <div className={cn("flex items-center justify-between h-9 px-3 border-b border-border bg-muted/30", className)}>
      <div className="flex items-center gap-2">
        <span className="text-sm font-medium">{title}</span>
        {count !== undefined && (
          <span className="text-xs text-muted-foreground">({count})</span>
        )}
      </div>
      {children && <div className="flex items-center gap-1">{children}</div>}
    </div>
  );
}
