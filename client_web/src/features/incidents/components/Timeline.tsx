"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { useAddTimelineEntry, useEditTimelineEntry } from "../hooks";
import {
  Emoji,
  ReactionSummaryResponse,
  TimelineEntryResponse,
} from "../types";
import { TeamMemberResponse, TeamRole } from "@/features/teams";
import { Button } from "@/shared/ui/Button";
import { ReactionBar } from "./ReactionBar";

interface TimelineProps {
  teamId: string;
  incidentId: string;
  entries: TimelineEntryResponse[];
  members: TeamMemberResponse[];
  currentUserId: string;
  currentUserRole: TeamRole | undefined;
  availableEmojis: Emoji[];
  reactions: ReactionSummaryResponse[];
}

const formatDate = (unixSeconds: number) =>
  new Date(unixSeconds * 1000).toLocaleString();

const submitOnEnter = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    e.currentTarget.form?.requestSubmit();
  }
};

const TimelineEntry = ({
  entry,
  members,
  canEdit,
  teamId,
  incidentId,
  availableEmojis,
  reactions,
}: {
  entry: TimelineEntryResponse;
  members: TeamMemberResponse[];
  canEdit: boolean;
  teamId: string;
  incidentId: string;
  availableEmojis: Emoji[];
  reactions: ReactionSummaryResponse[];
}) => {
  const t = useTranslations("incidents");
  const commonT = useTranslations("common");
  const [isEditing, setIsEditing] = useState(false);
  const {
    mutate: editEntry,
    isPending,
    error,
  } = useEditTimelineEntry(teamId, incidentId);

  const authorName = (authorId: string | null) => {
    if (!authorId) return t("unknownAuthor");
    return members.find((m) => m.user_id === authorId)?.username ?? authorId;
  };

  const handleSave = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const content = (
      e.currentTarget.elements.namedItem("content") as HTMLTextAreaElement
    ).value;
    editEntry(
      { entryId: entry.id, body: { content } },
      { onSuccess: () => setIsEditing(false) },
    );
  };

  return (
    <div className="flex flex-col gap-1 px-4 py-3 border-b border-[var(--color-border)] last:border-0">
      <div className="flex items-center justify-between">
        <span className="text-caption font-medium text-[var(--color-text-primary)]">
          {authorName(entry.author_id)}
        </span>
        <div className="flex items-center gap-2">
          <span className="text-caption text-[var(--color-text-muted)]">
            {formatDate(entry.created_at)}
            {entry.edited_at ? t("edited") : ""}
          </span>
          {canEdit && !isEditing && (
            <button
              type="button"
              onClick={() => setIsEditing(true)}
              className="text-caption text-[var(--color-accent)] hover:underline"
            >
              {t("edit")}
            </button>
          )}
        </div>
      </div>

      {isEditing ? (
        <form onSubmit={handleSave} className="flex flex-col gap-2 mt-1">
          {error && (
            <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
              {error.message}
            </div>
          )}
          <label htmlFor={`edit-content-${entry.id}`} className="sr-only">
            {t("editTimelineEntry")}
          </label>
          <textarea
            id={`edit-content-${entry.id}`}
            name="content"
            defaultValue={entry.content}
            rows={2}
            onKeyDown={submitOnEnter}
            className="w-full px-3 py-2 text-body rounded-md border bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] border-[var(--color-border)] focus:outline-none focus:border-[var(--color-accent)]"
          />
          <div className="flex gap-2">
            <Button type="submit" variant="primary" isLoading={isPending}>
              {t("save")}
            </Button>
            <Button
              type="button"
              variant="secondary"
              onClick={() => setIsEditing(false)}
            >
              {commonT("cancel")}
            </Button>
          </div>
        </form>
      ) : (
        <p className="text-body text-[var(--color-text-primary)]">
          {entry.content}
        </p>
      )}

      <ReactionBar
        teamId={teamId}
        incidentId={incidentId}
        entryId={entry.id}
        availableEmojis={availableEmojis}
        reactions={reactions}
      />
    </div>
  );
};

export const Timeline = ({
  teamId,
  incidentId,
  entries,
  members,
  currentUserId,
  currentUserRole,
  availableEmojis,
  reactions,
}: TimelineProps) => {
  const t = useTranslations("incidents");
  const {
    mutate: addEntry,
    isPending: adding,
    error: addError,
  } = useAddTimelineEntry(teamId, incidentId);

  const canAddEntry =
    currentUserRole === "responder" || currentUserRole === "manager";

  const handleAdd = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const content = (form.elements.namedItem("content") as HTMLTextAreaElement)
      .value;
    if (!content.trim()) return;

    addEntry(
      { content },
      {
        onSuccess: () => form.reset(),
      },
    );
  };

  return (
    <div className="flex flex-col gap-4">
      <h2 className="text-subtitle font-medium text-[var(--color-text-primary)]">
        {t("timeline")}
      </h2>

      {entries.length === 0 ? (
        <p className="text-body text-[var(--color-text-muted)]">
          {t("noTimelineEntriesYet")}
        </p>
      ) : (
        <div className="rounded-md border border-[var(--color-border)] overflow-hidden">
          {entries.map((entry) => (
            <TimelineEntry
              key={entry.id}
              entry={entry}
              members={members}
              canEdit={entry.author_id === currentUserId}
              teamId={teamId}
              incidentId={incidentId}
              availableEmojis={availableEmojis}
              reactions={reactions.filter((r) => r.entry_id === entry.id)}
            />
          ))}
        </div>
      )}

      {canAddEntry && (
        <form onSubmit={handleAdd} className="flex flex-col gap-2">
          {addError && (
            <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
              {addError.message}
            </div>
          )}
          <label
            htmlFor="content"
            className="text-body font-medium text-[var(--color-text-secondary)]"
          >
            {t("addTimelineEntry")}
          </label>
          <textarea
            id="content"
            name="content"
            rows={2}
            onKeyDown={submitOnEnter}
            className="w-full px-3 py-2 text-body rounded-md border bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] border-[var(--color-border)] placeholder:text-[var(--color-text-muted)] hover:border-[var(--color-border-strong)] focus:outline-none focus:border-[var(--color-accent)]"
          />
          <Button
            type="submit"
            variant="secondary"
            isLoading={adding}
            className="w-fit"
          >
            {t("addEntry")}
          </Button>
        </form>
      )}
    </div>
  );
};
