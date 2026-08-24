import { useTranslations } from "next-intl";
import { Circle, Eye, ArrowUpCircle, CheckCircle2 } from "lucide-react";
import { IncidentState } from "../types";
import { Badge, BadgeColor } from "@/shared/ui/Badge";

interface StateBadgeProps {
  state: IncidentState;
}

const stateConfig: Record<
  IncidentState,
  { icon: typeof Circle; color: BadgeColor }
> = {
  open: { icon: Circle, color: "accent" },
  acknowledged: { icon: Eye, color: "medium" },
  escalated: { icon: ArrowUpCircle, color: "danger" },
  resolved: { icon: CheckCircle2, color: "success" },
};

export const StateBadge = ({ state }: StateBadgeProps) => {
  const t = useTranslations("common.incidentStates");
  const { icon, color } = stateConfig[state];
  return <Badge label={t(state)} icon={icon} color={color} />;
};
