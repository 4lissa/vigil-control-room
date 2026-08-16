"use client";

import { use } from "react";
import Link from "next/link";
import { useMe } from "@/features/auth";
import { useTeamMembers } from "@/features/teams";
import {
  useRelease,
  useReleaseRealtime,
  useReleaseSteps,
} from "@/features/releases";
import { ReleaseStateBadge } from "@/features/releases/components/ReleaseStateBadge";
import { ReleaseActions } from "@/features/releases/components/ReleaseActions";
import { StepList } from "@/features/releases/components/StepList";

export default function ReleaseDetailPage({
  params,
}: {
  params: Promise<{ teamId: string; releaseId: string }>;
}) {
  const { teamId, releaseId } = use(params);
  const { data: user } = useMe();
  const { data: members } = useTeamMembers(teamId);
  const { data: release, isLoading } = useRelease(teamId, releaseId);
  const { data: steps } = useReleaseSteps(teamId, releaseId);

  useReleaseRealtime(teamId, releaseId);

  if (isLoading) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">Loading...</p>
    );
  }

  if (!release) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">
        Release not found.
      </p>
    );
  }

  const currentMember = members?.find((m) => m.user_id === user?.id);
  const canValidate =
    (currentMember?.role === "responder" ||
      currentMember?.role === "manager") &&
    (release.state === "created" || release.state === "in_progress");

  return (
    <div className="flex flex-col gap-8 max-w-2xl">
      <Link
        href={`/teams/${teamId}/releases`}
        className="text-caption text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] w-fit"
      >
        ← Releases
      </Link>

      <div className="flex flex-col gap-3">
        <ReleaseStateBadge state={release.state} />
        <h1 className="text-title font-medium text-[var(--color-text-primary)]">
          {release.name}
        </h1>
      </div>

      <ReleaseActions
        teamId={teamId}
        release={release}
        currentUserRole={currentMember?.role}
      />

      <StepList
        teamId={teamId}
        releaseId={releaseId}
        steps={steps ?? []}
        canValidate={canValidate}
      />
    </div>
  );
}
