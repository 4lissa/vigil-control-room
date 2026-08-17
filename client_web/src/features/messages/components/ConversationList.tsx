import Link from "next/link";
import { Contact } from "../hooks";
import { MessageResponse } from "../types";

interface ConversationListProps {
  conversations: MessageResponse[];
  contacts: Contact[];
  currentUserId: string;
}

const formatDate = (unixSeconds: number) =>
  new Date(unixSeconds * 1000).toLocaleString();

export const ConversationList = ({
  conversations,
  contacts,
  currentUserId,
}: ConversationListProps) => {
  if (conversations.length === 0) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">
        No conversations yet. Start one below.
      </p>
    );
  }

  return (
    <div className="rounded-md border border-[var(--color-border)] overflow-hidden">
      {conversations.map((message) => {
        const counterpartId =
          message.sender_id === currentUserId
            ? message.recipient_id
            : message.sender_id;
        const counterpartName =
          contacts.find((contact) => contact.user_id === counterpartId)
            ?.username ?? counterpartId;

        return (
          <Link
            key={message.id}
            href={`/messages/${counterpartId}`}
            className="flex items-center gap-3 px-4 py-3 border-b border-[var(--color-border)] last:border-0 hover:bg-[var(--color-bg-tertiary)] transition-colors"
          >
            <div className="w-7 h-7 shrink-0 rounded-full bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] flex items-center justify-center text-caption font-bold text-[var(--color-text-secondary)]">
              {counterpartName[0]?.toUpperCase()}
            </div>
            <div className="flex flex-col min-w-0">
              <span className="text-body text-[var(--color-text-primary)]">
                {counterpartName}
              </span>
              <span className="text-caption text-[var(--color-text-muted)] truncate">
                {message.content}
              </span>
            </div>
            <span className="ml-auto shrink-0 text-caption text-[var(--color-text-muted)]">
              {formatDate(message.created_at)}
            </span>
          </Link>
        );
      })}
    </div>
  );
};
