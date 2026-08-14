"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useTeams } from "@/features/teams";

export default function IncidentsPage() {
  const router = useRouter();
  const { data: teams } = useTeams();
  const firstTeam = teams?.[0];

  useEffect(() => {
    if (firstTeam) router.replace(`/teams/${firstTeam.id}/incidents`);
  }, [firstTeam, router]);

  if (firstTeam) return null;

  return (
    <div className="flex flex-col items-center justify-center h-full gap-4 text-center">
      <p className="text-subtitle text-[var(--color-text-secondary)]">
        You don’t belong to any team yet.
      </p>
      <p className="text-body text-[var(--color-text-muted)]">
        Create or join a team to get started.
      </p>
    </div>
  );
}
