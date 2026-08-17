"use client";

import { useAddReaction, useRemoveReaction } from "../hooks";
import { Emoji, ReactionSummaryResponse } from "../types";

interface ReactionBarProps {
  teamId: string;
  incidentId: string;
  entryId: string;
  availableEmojis: Emoji[];
  reactions: ReactionSummaryResponse[];
}

const EMOJI_DISPLAY: Record<Emoji, string> = {
  "+1": "👍",
  "-1": "👎",
  eyes: "👀",
  warning: "⚠️",
  check: "✅",
  fire: "🔥",
};

export const ReactionBar = ({
  teamId,
  incidentId,
  entryId,
  availableEmojis,
  reactions,
}: ReactionBarProps) => {
  const {
    mutate: addReaction,
    isPending: adding,
    error: addError,
  } = useAddReaction(teamId, incidentId);
  const {
    mutate: removeReaction,
    isPending: removing,
    error: removeError,
  } = useRemoveReaction(teamId, incidentId);

  const isBusy = adding || removing;
  const error = addError ?? removeError;

  const toggle = (emoji: Emoji, reacted: boolean) => {
    if (reacted) {
      removeReaction({ entryId, emoji });
    } else {
      addReaction({ entryId, emoji });
    }
  };

  return (
    <div className="flex flex-col gap-1 mt-1">
      <div className="flex items-center gap-1">
        {availableEmojis.map((emoji) => {
          const summary = reactions.find((r) => r.emoji === emoji);
          const reacted = summary?.reacted_by_me ?? false;

          const usernames = summary?.usernames ?? [];

          return (
            <div key={emoji} className="group relative">
              <button
                type="button"
                disabled={isBusy}
                aria-pressed={reacted}
                aria-label={
                  usernames.length > 0
                    ? `React with ${emoji} (${usernames.join(", ")})`
                    : `React with ${emoji}`
                }
                onClick={() => toggle(emoji, reacted)}
                className={`flex items-center gap-1 px-2 py-0.5 rounded-full border text-caption transition-colors disabled:opacity-50 disabled:cursor-wait ${
                  reacted
                    ? "border-[var(--color-accent)] bg-[var(--color-accent)]/10 text-[var(--color-accent)]"
                    : "border-[var(--color-border)] text-[var(--color-text-muted)] hover:border-[var(--color-border-strong)]"
                }`}
              >
                <span>{EMOJI_DISPLAY[emoji]}</span>
                {usernames.length > 0 && <span>{usernames.length}</span>}
              </button>

              {usernames.length > 0 && (
                <div className="pointer-events-none absolute bottom-full left-1/2 z-10 mb-1 hidden -translate-x-1/2 whitespace-nowrap rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] px-2 py-1 text-caption text-[var(--color-text-primary)] shadow-md group-hover:block">
                  {usernames.join(", ")}
                </div>
              )}
            </div>
          );
        })}
      </div>

      {error && (
        <p className="text-caption text-[var(--color-danger)]">
          {error.message}
        </p>
      )}
    </div>
  );
};
