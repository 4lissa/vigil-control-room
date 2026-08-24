"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { useBanMember, useKickMember } from "../hooks";
import { TeamMemberResponse, TeamRole } from "../types";
import { Button } from "@/shared/ui/Button";
import { Dialog } from "@/shared/ui/Dialog";

interface TeamMemberListProps {
  teamId: string;
  teamName: string;
  members: TeamMemberResponse[];
  currentUserId: string;
  currentUserRole?: TeamRole;
}

export const TeamMemberList = ({
  teamId,
  teamName,
  members,
  currentUserId,
  currentUserRole,
}: TeamMemberListProps) => {
  const t = useTranslations("teams");
  const commonT = useTranslations("common");
  const [kickTarget, setKickTarget] = useState<TeamMemberResponse | null>(null);
  const [banTarget, setBanTarget] = useState<TeamMemberResponse | null>(null);
  const [banUntil, setBanUntil] = useState("");

  const {
    mutate: kick,
    isPending: kicking,
    error: kickError,
  } = useKickMember(teamId);
  const {
    mutate: ban,
    isPending: banning,
    error: banError,
  } = useBanMember(teamId);

  const canModerate = currentUserRole === "manager";

  const closeBanDialog = () => {
    setBanTarget(null);
    setBanUntil("");
  };

  const handleKick = () => {
    if (!kickTarget) return;
    kick(kickTarget.user_id, { onSuccess: () => setKickTarget(null) });
  };

  const handleBan = () => {
    if (!banTarget) return;
    const until = banUntil
      ? Math.floor(new Date(banUntil).getTime() / 1000)
      : null;
    ban(
      { userId: banTarget.user_id, body: { until } },
      { onSuccess: closeBanDialog },
    );
  };

  return (
    <div className="flex flex-col gap-2">
      <h2 className="text-subtitle font-medium text-[var(--color-text-primary)]">
        {t("members", { count: members.length })}
      </h2>
      <div className="rounded-md border border-[var(--color-border)] overflow-hidden">
        {members.map((member) => (
          <div
            key={member.id}
            className="flex items-center justify-between px-4 py-3 border-b border-[var(--color-border)] last:border-0 hover:bg-[var(--color-bg-tertiary)] transition-colors"
          >
            <div className="flex items-center gap-3">
              <div className="w-7 h-7 rounded-full bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] flex items-center justify-center text-caption font-bold text-[var(--color-text-secondary)]">
                {(member.username ?? member.user_id)[0]?.toUpperCase()}
              </div>
              <span className="text-body text-[var(--color-text-primary)]">
                {member.username ?? member.user_id}
                {member.user_id === currentUserId && (
                  <span className="ml-2 text-caption text-[var(--color-text-muted)]">
                    {t("you")}
                  </span>
                )}
              </span>
            </div>
            <div className="flex items-center gap-3">
              <span className="text-caption text-[var(--color-text-muted)]">
                {commonT(`roles.${member.role}`)}
              </span>
              {canModerate && member.user_id !== currentUserId && (
                <div className="flex items-center gap-3">
                  <button
                    type="button"
                    onClick={() => setKickTarget(member)}
                    className="text-caption text-[var(--color-text-muted)] hover:text-[var(--color-danger)] hover:underline"
                  >
                    {t("kick")}
                  </button>
                  <button
                    type="button"
                    onClick={() => setBanTarget(member)}
                    className="text-caption text-[var(--color-text-muted)] hover:text-[var(--color-danger)] hover:underline"
                  >
                    {t("ban")}
                  </button>
                </div>
              )}
            </div>
          </div>
        ))}
      </div>

      <Dialog
        isOpen={!!kickTarget}
        onClose={() => setKickTarget(null)}
        title={t("kickMemberTitle")}
      >
        <div className="flex flex-col gap-4">
          {kickError && (
            <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
              {kickError.message}
            </div>
          )}
          <p className="text-body text-[var(--color-text-primary)]">
            {t("kickConfirm", {
              member: kickTarget?.username ?? kickTarget?.user_id ?? "",
              teamName,
            })}
          </p>
          <div className="flex gap-3">
            <Button variant="danger" isLoading={kicking} onClick={handleKick}>
              {t("kickButton", {
                member: kickTarget?.username ?? kickTarget?.user_id ?? "",
              })}
            </Button>
            <Button variant="secondary" onClick={() => setKickTarget(null)}>
              {commonT("cancel")}
            </Button>
          </div>
        </div>
      </Dialog>

      <Dialog
        isOpen={!!banTarget}
        onClose={closeBanDialog}
        title={t("banMemberTitle")}
      >
        <div className="flex flex-col gap-4">
          {banError && (
            <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
              {banError.message}
            </div>
          )}
          <p className="text-body text-[var(--color-text-primary)]">
            {t("banConfirm", {
              member: banTarget?.username ?? banTarget?.user_id ?? "",
              teamName,
            })}
          </p>
          <div className="flex flex-col gap-1">
            <label
              htmlFor="ban-until"
              className="text-body font-medium text-[var(--color-text-secondary)]"
            >
              {t("liftBanLabel")}
            </label>
            <input
              id="ban-until"
              type="datetime-local"
              value={banUntil}
              min={new Date().toISOString().slice(0, 16)}
              onChange={(e) => setBanUntil(e.target.value)}
              className="px-3 py-2 text-body rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-accent)]"
            />
          </div>
          <div className="flex gap-3">
            <Button variant="danger" isLoading={banning} onClick={handleBan}>
              {t("banButton", {
                member: banTarget?.username ?? banTarget?.user_id ?? "",
              })}
            </Button>
            <Button variant="secondary" onClick={closeBanDialog}>
              {commonT("cancel")}
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
};
