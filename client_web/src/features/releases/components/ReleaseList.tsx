import Link from "next/link";
import { ReleaseResponse } from "../types";
import { ReleaseStateBadge } from "./ReleaseStateBadge";

interface ReleaseListProps {
  teamId: string;
  releases: ReleaseResponse[];
}

export const ReleaseList = ({ teamId, releases }: ReleaseListProps) => {
  if (releases.length === 0) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">
        No releases yet.
      </p>
    );
  }

  return (
    <div className="rounded-md border border-[var(--color-border)] overflow-hidden">
      {releases.map((release) => (
        <Link
          key={release.id}
          href={`/teams/${teamId}/releases/${release.id}`}
          className="flex items-center justify-between gap-4 px-4 py-3 border-b border-[var(--color-border)] last:border-0 hover:bg-[var(--color-bg-tertiary)] transition-colors"
        >
          <span className="text-body text-[var(--color-text-primary)] truncate">
            {release.name}
          </span>
          <ReleaseStateBadge state={release.state} />
        </Link>
      ))}
    </div>
  );
};
