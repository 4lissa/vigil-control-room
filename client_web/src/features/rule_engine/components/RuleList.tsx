"use client";

import { useState } from "react";
import { useDeleteRule, useUpdateRule } from "../hooks";
import { RuleResponse } from "../types";
import { API_BASE_URL } from "@/shared/lib/api-client";
import { Badge } from "@/shared/ui/Badge";
import { Button } from "@/shared/ui/Button";
import { Dialog } from "@/shared/ui/Dialog";
import { CheckCircle2, CircleOff } from "lucide-react";

interface RuleListProps {
  teamId: string;
  rules: RuleResponse[];
}

export const RuleList = ({ teamId, rules }: RuleListProps) => {
  const [deleteTarget, setDeleteTarget] = useState<RuleResponse | null>(null);
  const { mutate: updateRule } = useUpdateRule(teamId);
  const {
    mutate: deleteRule,
    isPending: deleting,
    error: deleteError,
  } = useDeleteRule(teamId);

  if (rules.length === 0) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">No rules yet.</p>
    );
  }

  const handleDelete = () => {
    if (!deleteTarget) return;
    deleteRule(deleteTarget.id, { onSuccess: () => setDeleteTarget(null) });
  };

  return (
    <div className="flex flex-col gap-2">
      <div className="rounded-md border border-[var(--color-border)] overflow-hidden">
        {rules.map((rule) => (
          <div
            key={rule.id}
            className="flex items-center justify-between gap-4 px-4 py-3 border-b border-[var(--color-border)] last:border-0"
          >
            <div className="flex flex-col gap-1 min-w-0">
              <span className="text-body text-[var(--color-text-primary)] truncate">
                {rule.name}
              </span>
              <span className="text-caption text-[var(--color-text-muted)] truncate">
                {rule.trigger.service}/{rule.trigger.event} on{" "}
                {rule.trigger.filters["repository.full_name"] ?? "any repo"} →{" "}
                {rule.reaction.type}
              </span>
              {rule.trigger.service === "github" && (
                <code className="text-caption text-[var(--color-text-muted)] break-all select-all">
                  {API_BASE_URL}/webhooks/github/{rule.id}
                </code>
              )}
              {rule.reaction.type === "http_post" &&
                rule.reaction.payload.url && (
                  <code className="text-caption text-[var(--color-text-muted)] break-all select-all">
                    {rule.reaction.payload.url}
                  </code>
                )}
            </div>
            <div className="flex items-center gap-3 shrink-0">
              <button
                type="button"
                onClick={() =>
                  updateRule({
                    ruleId: rule.id,
                    body: { enabled: !rule.enabled },
                  })
                }
              >
                {rule.enabled ? (
                  <Badge label="Enabled" icon={CheckCircle2} color="success" />
                ) : (
                  <Badge label="Disabled" icon={CircleOff} color="neutral" />
                )}
              </button>
              <button
                type="button"
                onClick={() => setDeleteTarget(rule)}
                className="text-caption text-[var(--color-text-muted)] hover:text-[var(--color-danger)] hover:underline"
              >
                Delete
              </button>
            </div>
          </div>
        ))}
      </div>

      <Dialog
        isOpen={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        title="Delete rule"
      >
        <div className="flex flex-col gap-4">
          {deleteError && (
            <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
              {deleteError.message}
            </div>
          )}
          <p className="text-body text-[var(--color-text-primary)]">
            Delete <strong>{deleteTarget?.name}</strong>? The webhook on GitHub
            will keep sending events, but they will be rejected.
          </p>
          <div className="flex gap-3">
            <Button
              variant="danger"
              isLoading={deleting}
              onClick={handleDelete}
            >
              Delete {deleteTarget?.name}
            </Button>
            <Button variant="secondary" onClick={() => setDeleteTarget(null)}>
              Cancel
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
};
