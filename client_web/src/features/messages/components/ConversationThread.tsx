"use client";

import { useTranslations } from "next-intl";
import { useSendMessage } from "../hooks";
import { MessageResponse } from "../types";
import { Button } from "@/shared/ui/Button";

interface ConversationThreadProps {
  userId: string;
  otherUsername: string;
  currentUserId: string;
  messages: MessageResponse[];
}

const formatDate = (unixSeconds: number) =>
  new Date(unixSeconds * 1000).toLocaleString();

const submitOnEnter = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    e.currentTarget.form?.requestSubmit();
  }
};

export const ConversationThread = ({
  userId,
  otherUsername,
  currentUserId,
  messages,
}: ConversationThreadProps) => {
  const t = useTranslations("messages");
  const { mutate: send, isPending, error } = useSendMessage(userId);

  const handleSend = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const content = (form.elements.namedItem("content") as HTMLTextAreaElement)
      .value;
    if (!content.trim()) return;

    send({ content }, { onSuccess: () => form.reset() });
  };

  return (
    <div className="flex flex-col gap-4">
      {messages.length === 0 ? (
        <p className="text-body text-[var(--color-text-muted)]">
          {t("noMessagesYet", { username: otherUsername })}
        </p>
      ) : (
        <div className="rounded-md border border-[var(--color-border)] overflow-hidden">
          {messages.map((message) => (
            <div
              key={message.id}
              className="flex flex-col gap-1 px-4 py-3 border-b border-[var(--color-border)] last:border-0"
            >
              <div className="flex items-center justify-between">
                <span className="text-caption font-medium text-[var(--color-text-primary)]">
                  {message.sender_id === currentUserId
                    ? t("you")
                    : otherUsername}
                </span>
                <span className="text-caption text-[var(--color-text-muted)]">
                  {formatDate(message.created_at)}
                </span>
              </div>
              <p className="text-body text-[var(--color-text-primary)]">
                {message.content}
              </p>
            </div>
          ))}
        </div>
      )}

      <form onSubmit={handleSend} className="flex flex-col gap-2">
        {error && (
          <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
            {error.message}
          </div>
        )}
        <label
          htmlFor="content"
          className="text-body font-medium text-[var(--color-text-secondary)]"
        >
          {t("messageUser", { username: otherUsername })}
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
          variant="primary"
          isLoading={isPending}
          className="w-fit"
        >
          {t("send")}
        </Button>
      </form>
    </div>
  );
};
