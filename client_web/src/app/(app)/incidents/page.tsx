"use client";

import { useEffect } from "react";
import { useTranslations } from "next-intl";
import { useRouter } from "next/navigation";
import { useDefaultTeamId } from "@/features/teams";

export default function IncidentsPage() {
  const t = useTranslations("common");
  const router = useRouter();
  const defaultTeamId = useDefaultTeamId();

  useEffect(() => {
    if (defaultTeamId) router.replace(`/teams/${defaultTeamId}/incidents`);
  }, [defaultTeamId, router]);

  if (defaultTeamId) return null;

  return (
    <div className="flex flex-col items-center justify-center h-full gap-4 text-center">
      <p className="text-subtitle text-[var(--color-text-secondary)]">
        {t("noTeamYet")}
      </p>
      <p className="text-body text-[var(--color-text-muted)]">
        {t("createOrJoinToStart")}
      </p>
    </div>
  );
}
