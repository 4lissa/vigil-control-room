"use client";

import { useState } from "react";
import {
  useAcknowledgeIncident,
  useEscalateIncident,
  useResolveIncident,
  useAssignResponder,
} from "../hooks";
import { IncidentResponse, Severity } from "../types";
import { TeamMemberResponse, TeamRole } from "@/features/teams";
import { Button } from "@/shared/ui/Button";

interface IncidentActionsProps {
  teamId: string;
  incident: IncidentResponse;
  members: TeamMemberResponse[];
  currentUserRole: TeamRole | undefined;
}

export const IncidentActions = ({
  teamId,
  incident,
  members,
  currentUserRole,
}: IncidentActionsProps) => {
  const [escalateSeverity, setEscalateSeverity] = useState<Severity>(
    incident.severity,
  );
  const [assignedTo, setAssignedTo] = useState(incident.assigned_to ?? "");

  const {
    mutate: acknowledge,
    isPending: acknowledging,
    error: acknowledgeError,
  } = useAcknowledgeIncident(teamId, incident.id);
  const {
    mutate: escalate,
    isPending: escalating,
    error: escalateError,
  } = useEscalateIncident(teamId, incident.id);
  const {
    mutate: resolve,
    isPending: resolving,
    error: resolveError,
  } = useResolveIncident(teamId, incident.id);
  const {
    mutate: assign,
    isPending: assigning,
    error: assignError,
  } = useAssignResponder(teamId, incident.id);

  const error =
    acknowledgeError ?? escalateError ?? resolveError ?? assignError;

  const canRespond =
    currentUserRole === "responder" || currentUserRole === "manager";
  const canManage = currentUserRole === "manager";

  const canAcknowledge = canRespond && incident.state === "open";
  const canEscalate =
    canRespond &&
    (incident.state === "acknowledged" || incident.state === "escalated");
  const canResolve = canManage && incident.state !== "resolved";
  const canAssign = canManage && incident.state !== "resolved";

  if (!canAcknowledge && !canEscalate && !canResolve && !canAssign) {
    return null;
  }

  return (
    <div className="flex flex-col gap-4">
      <h2 className="text-subtitle font-medium text-[var(--color-text-primary)]">
        Actions
      </h2>

      {error && (
        <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
          {error.message}
        </div>
      )}

      <div className="flex flex-wrap items-center gap-3">
        {canAcknowledge && (
          <Button
            variant="secondary"
            isLoading={acknowledging}
            onClick={() => acknowledge()}
          >
            Acknowledge
          </Button>
        )}

        {canEscalate && (
          <div className="flex flex-col gap-1">
            <label
              htmlFor="escalate-severity"
              className="text-caption font-medium text-[var(--color-text-secondary)]"
            >
              New severity
            </label>
            <div className="flex items-center gap-2">
              <select
                id="escalate-severity"
                value={escalateSeverity}
                onChange={(e) =>
                  setEscalateSeverity(e.target.value as Severity)
                }
                className="px-3 py-2 text-body rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-accent)]"
              >
                <option value="low">Low</option>
                <option value="medium">Medium</option>
                <option value="high">High</option>
                <option value="critical">Critical</option>
              </select>
              <Button
                variant="secondary"
                isLoading={escalating}
                onClick={() => escalate({ severity: escalateSeverity })}
              >
                Escalate
              </Button>
            </div>
          </div>
        )}

        {canResolve && (
          <Button
            variant="primary"
            isLoading={resolving}
            onClick={() => resolve()}
          >
            Resolve
          </Button>
        )}
      </div>

      {canAssign && (
        <div className="flex flex-col gap-1">
          <label
            htmlFor="assign-to"
            className="text-caption font-medium text-[var(--color-text-secondary)]"
          >
            Assign to
          </label>
          <div className="flex items-center gap-2">
            <select
              id="assign-to"
              value={assignedTo}
              onChange={(e) => setAssignedTo(e.target.value)}
              className="px-3 py-2 text-body rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-accent)]"
            >
              <option value="">Unassigned</option>
              {members.map((m) => (
                <option key={m.id} value={m.user_id}>
                  {m.username ?? m.user_id}
                </option>
              ))}
            </select>
            <Button
              variant="secondary"
              isLoading={assigning}
              onClick={() => assign({ user_id: assignedTo || null })}
            >
              Assign
            </Button>
          </div>
        </div>
      )}
    </div>
  );
};
