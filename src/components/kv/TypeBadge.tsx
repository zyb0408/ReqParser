import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";

const TYPE_STYLES: Record<string, { label: string; className: string }> = {
  jwt: {
    label: "JWT",
    className: "text-[var(--type-jwt)] bg-[var(--type-jwt-bg)]",
  },
  base64: {
    label: "B64",
    className: "text-[var(--type-base64)] bg-[var(--type-base64-bg)]",
  },
  timestamp: {
    label: "TIME",
    className: "text-[var(--type-timestamp)] bg-[var(--type-timestamp-bg)]",
  },
  json: {
    label: "JSON",
    className: "text-[var(--type-json)] bg-[var(--type-json-bg)]",
  },
  url_encoded: {
    label: "URL",
    className: "text-[var(--type-url)] bg-[var(--type-url-bg)]",
  },
  compound: {
    label: "ENC",
    className: "text-[var(--type-encoded)] bg-[var(--type-encoded-bg)]",
  },
};

interface TypeBadgeProps {
  type?: string;
}

export function TypeBadge({ type }: TypeBadgeProps) {
  if (!type || type === "plain" || !TYPE_STYLES[type]) return null;

  const style = TYPE_STYLES[type];
  return (
    <Badge
      variant="outline"
      className={cn(
        "text-[10px] font-mono font-semibold px-1.5 py-0 h-[18px] rounded-sm border-none animate-in fade-in-0 zoom-in-95 duration-150 shadow-[inset_0_0_0_1px_oklch(1_0_0/8%)]",
        style.className
      )}
    >
      {style.label}
    </Badge>
  );
}
