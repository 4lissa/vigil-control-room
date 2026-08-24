import { useTranslations } from "next-intl";
import { ArrowDown, Minus, ArrowUp, Flame } from "lucide-react";
import { Severity } from "../types";
import { Badge, BadgeColor } from "@/shared/ui/Badge";

interface SeverityBadgeProps {
  severity: Severity;
}

const severityConfig: Record<
  Severity,
  { icon: typeof ArrowDown; color: BadgeColor }
> = {
  low: { icon: ArrowDown, color: "neutral" },
  medium: { icon: Minus, color: "medium" },
  high: { icon: ArrowUp, color: "warning" },
  critical: { icon: Flame, color: "danger" },
};

export const SeverityBadge = ({ severity }: SeverityBadgeProps) => {
  const t = useTranslations("common.severities");
  const { icon, color } = severityConfig[severity];
  return <Badge label={t(severity)} icon={icon} color={color} />;
};
