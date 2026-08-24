"use client";

import { useTranslations } from "next-intl";
import { useTeamBans, useUnbanMember } from "../hooks";

interface BannedMembersListProps {
  teamId: string;
}

const formatDate = (unixSeconds: number) =>
  new Date(unixSeconds * 1000).toLocaleString();

export const BannedMembersList = ({ teamId }: BannedMembersListProps) => {
  const t = useTranslations("teams");
  const { data: bans, isLoading } = useTeamBans(teamId);
  const { mutate: unban, isPending, error } = useUnbanMember(teamId);

  if (isLoading || !bans || bans.length === 0) {
    return null;
  }

  return (
    <div className="flex flex-col gap-2">
      <h2 className="text-subtitle font-medium text-[var(--color-text-primary)]">
        {t("bannedMembers", { count: bans.length })}
      </h2>
      {error && (
        <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
          {error.message}
        </div>
      )}
      <div className="rounded-md border border-[var(--color-border)] overflow-hidden">
        {bans.map((ban) => (
          <div
            key={ban.id}
            className="flex items-center justify-between px-4 py-3 border-b border-[var(--color-border)] last:border-0"
          >
            <div className="flex flex-col">
              <span className="text-body text-[var(--color-text-primary)]">
                {ban.username}
              </span>
              <span className="text-caption text-[var(--color-text-muted)]">
                {ban.until
                  ? t("until", { date: formatDate(ban.until) })
                  : t("permanentBan")}
              </span>
            </div>
            <button
              type="button"
              disabled={isPending}
              onClick={() => unban(ban.user_id)}
              className="text-caption text-[var(--color-accent)] hover:underline disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {t("unban")}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
};
