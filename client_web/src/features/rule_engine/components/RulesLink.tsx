"use client";

import { useTranslations } from "next-intl";
import { Button } from "@/shared/ui/Button";

interface RulesLinkProps {
  teamId: string;
}

export const RulesLink = ({ teamId }: RulesLinkProps) => {
  const t = useTranslations("rule_engine");

  return (
    <Button
      variant="secondary"
      href={`/teams/${teamId}/rules`}
      className="w-fit"
    >
      {t("automationRules")}
    </Button>
  );
};
