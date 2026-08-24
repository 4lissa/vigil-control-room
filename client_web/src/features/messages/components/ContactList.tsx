"use client";

import { useTranslations } from "next-intl";
import Link from "next/link";
import { Contact } from "../hooks";

interface ContactListProps {
  contacts: Contact[];
  emptyMessage?: string;
}

export const ContactList = ({ contacts, emptyMessage }: ContactListProps) => {
  const t = useTranslations("messages");

  if (contacts.length === 0) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">
        {emptyMessage ?? t("noTeammatesYet")}
      </p>
    );
  }

  return (
    <div className="rounded-md border border-[var(--color-border)] overflow-hidden">
      {contacts.map((contact) => (
        <Link
          key={contact.user_id}
          href={`/messages/${contact.user_id}`}
          className="flex items-center gap-3 px-4 py-3 border-b border-[var(--color-border)] last:border-0 hover:bg-[var(--color-bg-tertiary)] transition-colors"
        >
          <div className="w-7 h-7 rounded-full bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] flex items-center justify-center text-caption font-bold text-[var(--color-text-secondary)]">
            {contact.username[0]?.toUpperCase()}
          </div>
          <span className="text-body text-[var(--color-text-primary)]">
            {contact.username}
          </span>
        </Link>
      ))}
    </div>
  );
};
