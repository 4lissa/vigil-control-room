"use client";

import { useState } from "react";
import { useGenerateInviteCode, useTransferManager } from "../hooks";
import { TeamMemberResponse } from "../types";
import { Button } from "@/shared/ui/Button";

interface ManagerActionsProps {
  teamId: string;
  members: TeamMemberResponse[];
  currentUserId: string;
}

export const ManagerActions = ({
  teamId,
  members,
  currentUserId,
}: ManagerActionsProps) => {
  const [inviteCode, setInviteCode] = useState<string | null>(null);
  const [selectedUserId, setSelectedUserId] = useState("");
  const [showTransfer, setShowTransfer] = useState(false);

  const { mutate: generateCode, isPending: generatingCode } =
    useGenerateInviteCode(teamId);
  const { mutate: transfer, isPending: transferring } =
    useTransferManager(teamId);

  const otherMembers = members.filter((m) => m.user_id !== currentUserId);

  const handleGenerateCode = () => {
    generateCode(undefined, {
      onSuccess: (data) => setInviteCode(data.invitation_code),
    });
  };

  const handleTransfer = () => {
    if (!selectedUserId) return;
    transfer(
      { user_id: selectedUserId },
      { onSuccess: () => setShowTransfer(false) },
    );
  };

  return (
    <div className="flex flex-col gap-6">
      <h2 className="text-subtitle font-medium text-[var(--color-text-primary)]">
        Manager actions
      </h2>

      <div className="flex flex-col gap-2">
        <p className="text-body text-[var(--color-text-secondary)]">
          Invitation code
        </p>
        {inviteCode ? (
          <div className="flex items-center gap-3">
            <code className="px-3 py-2 rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-body text-[var(--color-accent)] font-mono">
              {inviteCode}
            </code>
            <Button
              variant="secondary"
              onClick={() => navigator.clipboard.writeText(inviteCode)}
            >
              Copy
            </Button>
          </div>
        ) : (
          <Button
            variant="secondary"
            isLoading={generatingCode}
            onClick={handleGenerateCode}
            className="w-fit"
          >
            Generate invite code
          </Button>
        )}
      </div>

      <div className="flex flex-col gap-2">
        <p className="text-body text-[var(--color-text-secondary)]">
          Transfer manager role
        </p>
        {!showTransfer ? (
          <Button
            variant="danger"
            onClick={() => setShowTransfer(true)}
            className="w-fit"
          >
            Transfer manager
          </Button>
        ) : (
          <div className="flex flex-col gap-3">
            <select
              value={selectedUserId}
              onChange={(e) => setSelectedUserId(e.target.value)}
              className="px-3 py-2 text-body rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-accent)]"
            >
              <option value="">Select a member</option>
              {otherMembers.map((m) => (
                <option key={m.id} value={m.user_id}>
                  {m.username ?? m.user_id}
                </option>
              ))}
            </select>
            <div className="flex gap-3">
              <Button
                variant="danger"
                isLoading={transferring}
                onClick={handleTransfer}
                disabled={!selectedUserId}
              >
                Confirm transfer
              </Button>
              <Button
                variant="secondary"
                onClick={() => setShowTransfer(false)}
              >
                Cancel
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
