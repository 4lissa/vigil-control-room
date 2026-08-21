"use client";

import { use, useEffect, useState } from "react";
import { useMe } from "@/features/auth";
import { useTeam, useTeamMembers, setLastTeamId } from "@/features/teams";
import { useRules, useRuleEventsFeed } from "@/features/rule_engine";
import { RuleList } from "@/features/rule_engine/components/RuleList";
import { CreateRuleForm } from "@/features/rule_engine/components/CreateRuleForm";
import { Button } from "@/shared/ui/Button";
import { Dialog } from "@/shared/ui/Dialog";

export default function TeamRulesPage({
  params,
}: {
  params: Promise<{ teamId: string }>;
}) {
  const { teamId } = use(params);
  const { data: user } = useMe();
  const { data: team, isLoading: teamLoading } = useTeam(teamId);
  const { data: members } = useTeamMembers(teamId);
  const { data: rules, isLoading: rulesLoading } = useRules(teamId);
  const [createOpen, setCreateOpen] = useState(false);
  const events = useRuleEventsFeed(teamId);

  useEffect(() => {
    setLastTeamId(teamId);
  }, [teamId]);

  const currentMember = members?.find((m) => m.user_id === user?.id);
  const isManager = currentMember?.role === "manager";

  if (teamLoading) {
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

  if (!isManager) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">
        Only the team&apos;s manager can configure automation rules.
      </p>
    );
  }

  return (
    <div className="flex flex-col gap-6 max-w-2xl">
      <div className="flex items-center justify-between">
        <h1 className="text-title font-medium text-[var(--color-text-primary)]">
          Automation rules - {team.name}
        </h1>
        <Button variant="primary" onClick={() => setCreateOpen(true)}>
          Create rule
        </Button>
      </div>

      {rulesLoading || !rules ? (
        <p className="text-body text-[var(--color-text-muted)]">Loading...</p>
      ) : (
        <RuleList teamId={teamId} rules={rules} />
      )}

      {events.length > 0 && (
        <div className="flex flex-col gap-2">
          <h2 className="text-subtitle font-medium text-[var(--color-text-primary)]">
            Live feed
          </h2>
          <div className="rounded-md border border-[var(--color-border)] overflow-hidden">
            {events.map((event) => (
              <div
                key={event.id}
                className="flex items-center justify-between gap-4 px-4 py-2 border-b border-[var(--color-border)] last:border-0"
              >
                <span className="text-body text-[var(--color-text-primary)] truncate">
                  {event.ruleName}
                </span>
                <span
                  className={`text-caption ${
                    event.status === "triggered"
                      ? "text-[var(--color-success)]"
                      : "text-[var(--color-danger)]"
                  }`}
                >
                  {event.detail}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      <Dialog
        isOpen={createOpen}
        onClose={() => setCreateOpen(false)}
        title="Create a rule"
      >
        <CreateRuleForm
          teamId={teamId}
          onSuccess={() => setCreateOpen(false)}
        />
      </Dialog>
    </div>
  );
}
