"use client";

import { use, useEffect, useState } from "react";
import { useMe } from "@/features/auth";
import { useTeam, useTeamMembers, setLastTeamId } from "@/features/teams";
import { useIncidents, useIncidentsRealtime } from "@/features/incidents";
import { IncidentList } from "@/features/incidents/components/IncidentList";
import { CreateIncidentForm } from "@/features/incidents/components/CreateIncidentForm";
import { Button } from "@/shared/ui/Button";
import { Dialog } from "@/shared/ui/Dialog";

export default function TeamIncidentsPage({
  params,
}: {
  params: Promise<{ teamId: string }>;
}) {
  const { teamId } = use(params);
  const { data: user } = useMe();
  const { data: team, isLoading: teamLoading } = useTeam(teamId);
  const { data: members } = useTeamMembers(teamId);
  const { data: incidents, isLoading: incidentsLoading } = useIncidents(teamId);
  const [createOpen, setCreateOpen] = useState(false);

  useEffect(() => {
    setLastTeamId(teamId);
  }, [teamId]);

  useIncidentsRealtime(teamId);

  if (teamLoading || incidentsLoading) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">Loading...</p>
    );
  }

  if (!team || !incidents) {
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
          Incidents - {team.name}
        </h1>
        {isManager && (
          <Button variant="primary" onClick={() => setCreateOpen(true)}>
            Create incident
          </Button>
        )}
      </div>

      <IncidentList teamId={teamId} incidents={incidents} />

      <Dialog
        isOpen={createOpen}
        onClose={() => setCreateOpen(false)}
        title="Create an incident"
      >
        <CreateIncidentForm
          teamId={teamId}
          onSuccess={() => setCreateOpen(false)}
        />
      </Dialog>
    </div>
  );
}
