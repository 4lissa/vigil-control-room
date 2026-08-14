"use client";

import { use, useEffect } from "react";
import { useTeam, setLastTeamId } from "@/features/teams";

export default function TeamReleasesPage({
  params,
}: {
  params: Promise<{ teamId: string }>;
}) {
  const { teamId } = use(params);
  const { data: team, isLoading } = useTeam(teamId);

  useEffect(() => {
    setLastTeamId(teamId);
  }, [teamId]);

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
        Releases - {team.name}
      </h1>
    </div>
  );
}
