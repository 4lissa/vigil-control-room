import { ArrowDown, Minus, ArrowUp, Flame } from "lucide-react";
import { Severity } from "../types";
import { Badge, BadgeColor } from "@/shared/ui/Badge";

interface SeverityBadgeProps {
  severity: Severity;
}

const severityConfig: Record<
  Severity,
  { label: string; icon: typeof ArrowDown; color: BadgeColor }
> = {
  low: { label: "Low", icon: ArrowDown, color: "neutral" },
  medium: { label: "Medium", icon: Minus, color: "medium" },
  high: { label: "High", icon: ArrowUp, color: "warning" },
  critical: { label: "Critical", icon: Flame, color: "danger" },
};

export const SeverityBadge = ({ severity }: SeverityBadgeProps) => {
  const { label, icon, color } = severityConfig[severity];
  return <Badge label={label} icon={icon} color={color} />;
};
