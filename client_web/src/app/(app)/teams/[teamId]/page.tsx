"use client";

import { use } from "react";
import { useMe } from "@/features/auth";
import { useTeam, useTeamMembers } from "@/features/teams";
import { TeamMemberList } from "@/features/teams/components/TeamMemberList";
import { ManagerActions } from "@/features/teams/components/ManagerActions";

export default function TeamPage({
  params,
}: {
  params: Promise<{ teamId: string }>;
}) {
  const { teamId } = use(params);
  const { data: user } = useMe();
  const { data: team, isLoading: teamLoading } = useTeam(teamId);
  const { data: members, isLoading: membersLoading } = useTeamMembers(teamId);

  if (teamLoading || membersLoading) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">Loading...</p>
    );
  }

  if (!team || !members) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">
        Team not found.
      </p>
    );
  }

  const currentMember = members.find((m) => m.user_id === user?.id);
  const isManager = currentMember?.role === "manager";

  return (
    <div className="flex flex-col gap-8 max-w-2xl">
      <div className="flex items-end gap-2">
        <h1 className="text-title font-medium leading-none text-[var(--color-text-primary)]">
          {team.name}
        </h1>
        {currentMember && (
          <p className="text-body leading-none text-[var(--color-text-muted)] capitalize">
            - {currentMember.role}
          </p>
        )}
      </div>

      <TeamMemberList members={members} currentUserId={user?.id ?? ""} />

      {isManager && (
        <ManagerActions
          teamId={teamId}
          members={members}
          currentUserId={user?.id ?? ""}
        />
      )}
    </div>
  );
}
