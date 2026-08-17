"use client";

import { use } from "react";
import Link from "next/link";
import { useMe } from "@/features/auth";
import {
  useContacts,
  useConversation,
  useMessagesRealtime,
} from "@/features/messages";
import { ConversationThread } from "@/features/messages/components/ConversationThread";

export default function ConversationPage({
  params,
}: {
  params: Promise<{ userId: string }>;
}) {
  const { userId } = use(params);
  const { data: user } = useMe();
  const { contacts } = useContacts(user?.id);
  const { data: messages, isLoading, isError, error } = useConversation(userId);

  const otherUsername =
    contacts.find((contact) => contact.user_id === userId)?.username ?? userId;

  useMessagesRealtime(user?.username);

  return (
    <div className="flex flex-col gap-6 max-w-2xl">
      <Link
        href="/messages"
        className="text-caption text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] w-fit"
      >
        ← Messages
      </Link>

      <h1 className="text-title font-medium text-[var(--color-text-primary)]">
        {otherUsername}
      </h1>

      {isLoading ? (
        <p className="text-body text-[var(--color-text-muted)]">Loading...</p>
      ) : isError ? (
        <p className="text-body text-[var(--color-danger)]">{error.message}</p>
      ) : (
        <ConversationThread
          userId={userId}
          otherUsername={otherUsername}
          currentUserId={user?.id ?? ""}
          messages={messages ?? []}
        />
      )}
    </div>
  );
}
