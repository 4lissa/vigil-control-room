"use client";

import { use, useEffect } from "react";
import { useMe } from "@/features/auth";
import { useTeam, useTeamMembers, setLastTeamId } from "@/features/teams";
import { TeamMemberList } from "@/features/teams/components/TeamMemberList";
import { ManagerActions } from "@/features/teams/components/ManagerActions";
import { BannedMembersList } from "@/features/teams/components/BannedMembersList";
import { RulesLink } from "@/features/rule_engine/components/RulesLink";

export default function TeamPage({
  params,
}: {
  params: Promise<{ teamId: string }>;
}) {
  const { teamId } = use(params);
  const { data: user } = useMe();
  const { data: team, isLoading: teamLoading } = useTeam(teamId);
  const { data: members, isLoading: membersLoading } = useTeamMembers(teamId);

  useEffect(() => {
    setLastTeamId(teamId);
  }, [teamId]);

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

      <TeamMemberList
        teamId={teamId}
        teamName={team.name}
        members={members}
        currentUserId={user?.id ?? ""}
        currentUserRole={currentMember?.role}
      />

      {isManager && (
        <>
          <BannedMembersList teamId={teamId} />
          <ManagerActions
            teamId={teamId}
            members={members}
            currentUserId={user?.id ?? ""}
          />
          <RulesLink teamId={teamId} />
        </>
      )}
    </div>
  );
}
