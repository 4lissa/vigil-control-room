import { Circle, PlayCircle, CheckCircle2, XCircle, Lock } from "lucide-react";
import { ReleaseState } from "../types";
import { Badge, BadgeColor } from "@/shared/ui/Badge";

interface ReleaseStateBadgeProps {
  state: ReleaseState;
}

const stateConfig: Record<
  ReleaseState,
  { label: string; icon: typeof Circle; color: BadgeColor }
> = {
  created: { label: "Created", icon: Circle, color: "neutral" },
  in_progress: { label: "In progress", icon: PlayCircle, color: "accent" },
  completed: { label: "Completed", icon: CheckCircle2, color: "success" },
  cancelled: { label: "Cancelled", icon: XCircle, color: "neutral" },
  blocked: { label: "Blocked", icon: Lock, color: "danger" },
};

export const ReleaseStateBadge = ({ state }: ReleaseStateBadgeProps) => {
  const { label, icon, color } = stateConfig[state];
  return <Badge label={label} icon={icon} color={color} />;
};
