"use client";

import { use } from "react";
import Link from "next/link";
import { Eye } from "lucide-react";
import { useMe } from "@/features/auth";
import { useTeamMembers } from "@/features/teams";
import {
  useIncident,
  useIncidentRealtime,
  useTimelineEntries,
} from "@/features/incidents";
import { useRelease } from "@/features/releases";
import { StateBadge } from "@/features/incidents/components/StateBadge";
import { SeverityBadge } from "@/features/incidents/components/SeverityBadge";
import { IncidentActions } from "@/features/incidents/components/IncidentActions";
import { Timeline } from "@/features/incidents/components/Timeline";

export default function IncidentDetailPage({
  params,
}: {
  params: Promise<{ teamId: string; incidentId: string }>;
}) {
  const { teamId, incidentId } = use(params);
  const { data: user } = useMe();
  const { data: members } = useTeamMembers(teamId);
  const { data: incident, isLoading } = useIncident(teamId, incidentId);
  const { data: timeline } = useTimelineEntries(teamId, incidentId);
  const { data: linkedRelease } = useRelease(
    teamId,
    incident?.release_id ?? "",
  );
  const { watchers } = useIncidentRealtime(teamId, incidentId);

  if (isLoading) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">Loading...</p>
    );
  }

  if (!incident) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">
        Incident not found.
      </p>
    );
  }

  const currentMember = members?.find((m) => m.user_id === user?.id);
  const assignedMember = members?.find(
    (m) => m.user_id === incident.assigned_to,
  );

  return (
    <div className="flex flex-col gap-8 max-w-2xl">
      <Link
        href={`/teams/${teamId}/incidents`}
        className="text-caption text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] w-fit"
      >
        ← Incidents
      </Link>

      <div className="flex flex-col gap-3">
        <div className="flex items-center gap-2">
          <SeverityBadge severity={incident.severity} />
          <StateBadge state={incident.state} />
        </div>
        <h1 className="text-title font-medium text-[var(--color-text-primary)]">
          {incident.title}
        </h1>
        {incident.description && (
          <p className="text-body text-[var(--color-text-secondary)]">
            {incident.description}
          </p>
        )}
        <p className="text-caption text-[var(--color-text-muted)]">
          Assigned to:{" "}
          {assignedMember?.username ?? incident.assigned_to ?? "Unassigned"}
        </p>
        {linkedRelease && (
          <p className="text-caption text-[var(--color-text-muted)]">
            Linked release:{" "}
            <Link
              href={`/teams/${teamId}/releases/${linkedRelease.id}`}
              className="text-[var(--color-accent)] hover:underline"
            >
              {linkedRelease.name}
            </Link>
          </p>
        )}
        {watchers.length > 0 && (
          <p className="flex items-center gap-1 text-caption text-[var(--color-text-muted)]">
            <Eye size={12} />
            {watchers.join(", ")}
          </p>
        )}
      </div>

      <IncidentActions
        key={`${incident.severity}-${incident.assigned_to}`}
        teamId={teamId}
        incident={incident}
        members={members ?? []}
        currentUserRole={currentMember?.role}
      />

      <Timeline
        teamId={teamId}
        incidentId={incidentId}
        entries={timeline ?? []}
        members={members ?? []}
        currentUserId={user?.id ?? ""}
        currentUserRole={currentMember?.role}
      />
    </div>
  );
}
