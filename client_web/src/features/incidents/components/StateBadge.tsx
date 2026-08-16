import { Circle, Eye, ArrowUpCircle, CheckCircle2 } from "lucide-react";
import { IncidentState } from "../types";
import { Badge, BadgeColor } from "@/shared/ui/Badge";

interface StateBadgeProps {
  state: IncidentState;
}

const stateConfig: Record<
  IncidentState,
  { label: string; icon: typeof Circle; color: BadgeColor }
> = {
  open: { label: "Open", icon: Circle, color: "accent" },
  acknowledged: { label: "Acknowledged", icon: Eye, color: "medium" },
  escalated: { label: "Escalated", icon: ArrowUpCircle, color: "danger" },
  resolved: { label: "Resolved", icon: CheckCircle2, color: "success" },
};

export const StateBadge = ({ state }: StateBadgeProps) => {
  const { label, icon, color } = stateConfig[state];
  return <Badge label={label} icon={icon} color={color} />;
};
