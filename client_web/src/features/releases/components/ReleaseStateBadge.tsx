import { useTranslations } from "next-intl";
import { Circle, PlayCircle, CheckCircle2, XCircle, Lock } from "lucide-react";
import { ReleaseState } from "../types";
import { Badge, BadgeColor } from "@/shared/ui/Badge";

interface ReleaseStateBadgeProps {
  state: ReleaseState;
}

const stateConfig: Record<
  ReleaseState,
  { icon: typeof Circle; color: BadgeColor }
> = {
  created: { icon: Circle, color: "neutral" },
  in_progress: { icon: PlayCircle, color: "accent" },
  completed: { icon: CheckCircle2, color: "success" },
  cancelled: { icon: XCircle, color: "neutral" },
  blocked: { icon: Lock, color: "danger" },
};

export const ReleaseStateBadge = ({ state }: ReleaseStateBadgeProps) => {
  const t = useTranslations("common.releaseStates");
  const { icon, color } = stateConfig[state];
  return <Badge label={t(state)} icon={icon} color={color} />;
};
