"use client";

import { use } from "react";
import { useTeam } from "@/features/teams";

export default function TeamIncidentsPage({
  params,
}: {
  params: Promise<{ teamId: string }>;
}) {
  const { teamId } = use(params);
  const { data: team, isLoading } = useTeam(teamId);

  if (isLoading) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">Loading...</p>
    );
  }

  if (!team) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">
        Team not found.
      </p>
    );
  }

  return (
    <div>
      <h1 className="text-title font-medium text-[var(--color-text-primary)]">
        Incidents - {team.name}
      </h1>
    </div>
  );
}
