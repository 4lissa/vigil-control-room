import Link from "next/link";
import { IncidentResponse } from "../types";
import { StateBadge } from "./StateBadge";
import { SeverityBadge } from "./SeverityBadge";

interface IncidentListProps {
  teamId: string;
  incidents: IncidentResponse[];
}

export const IncidentList = ({ teamId, incidents }: IncidentListProps) => {
  if (incidents.length === 0) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">
        No incidents yet.
      </p>
    );
  }

  return (
    <div className="rounded-md border border-[var(--color-border)] overflow-hidden">
      {incidents.map((incident) => (
        <Link
          key={incident.id}
          href={`/teams/${teamId}/incidents/${incident.id}`}
          className="flex items-center justify-between gap-4 px-4 py-3 border-b border-[var(--color-border)] last:border-0 hover:bg-[var(--color-bg-tertiary)] transition-colors"
        >
          <span className="text-body text-[var(--color-text-primary)] truncate">
            {incident.title}
          </span>
          <div className="flex items-center gap-2 shrink-0">
            <SeverityBadge severity={incident.severity} />
            <StateBadge state={incident.state} />
          </div>
        </Link>
      ))}
    </div>
  );
};
