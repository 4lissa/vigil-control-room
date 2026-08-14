import { TeamMemberResponse } from "../types";

interface TeamMemberListProps {
  members: TeamMemberResponse[];
  currentUserId: string;
}

export const TeamMemberList = ({
  members,
  currentUserId,
}: TeamMemberListProps) => {
  return (
    <div className="flex flex-col gap-2">
      <h2 className="text-subtitle font-medium text-[var(--color-text-primary)]">
        Members ({members.length})
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
                    (you)
                  </span>
                )}
              </span>
            </div>
            <span className="text-caption text-[var(--color-text-muted)] capitalize">
              {member.role}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
};
