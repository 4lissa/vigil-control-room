"use client";

import { use, useEffect, useState } from "react";
import { useMe } from "@/features/auth";
import { useTeam, useTeamMembers, setLastTeamId } from "@/features/teams";
import { useReleases, useReleasesRealtime } from "@/features/releases";
import { ReleaseList } from "@/features/releases/components/ReleaseList";
import { CreateReleaseForm } from "@/features/releases/components/CreateReleaseForm";
import { Button } from "@/shared/ui/Button";
import { Dialog } from "@/shared/ui/Dialog";

export default function TeamReleasesPage({
  params,
}: {
  params: Promise<{ teamId: string }>;
}) {
  const { teamId } = use(params);
  const { data: user } = useMe();
  const { data: team, isLoading: teamLoading } = useTeam(teamId);
  const { data: members } = useTeamMembers(teamId);
  const { data: releases, isLoading: releasesLoading } = useReleases(teamId);
  const [createOpen, setCreateOpen] = useState(false);

  useEffect(() => {
    setLastTeamId(teamId);
  }, [teamId]);

  useReleasesRealtime(teamId);

  if (teamLoading || releasesLoading) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">Loading...</p>
    );
  }

  if (!team || !releases) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">
        Team not found.
      </p>
    );
  }

  const currentMember = members?.find((m) => m.user_id === user?.id);
  const isManager = currentMember?.role === "manager";

  return (
    <div className="flex flex-col gap-6 max-w-2xl">
      <div className="flex items-center justify-between">
        <h1 className="text-title font-medium text-[var(--color-text-primary)]">
          Releases - {team.name}
        </h1>
        {isManager && (
          <Button variant="primary" onClick={() => setCreateOpen(true)}>
            Create release
          </Button>
        )}
      </div>

      <ReleaseList teamId={teamId} releases={releases} />

      <Dialog
        isOpen={createOpen}
        onClose={() => setCreateOpen(false)}
        title="Create a release"
      >
        <CreateReleaseForm
          teamId={teamId}
          onSuccess={() => setCreateOpen(false)}
        />
      </Dialog>
    </div>
  );
}
