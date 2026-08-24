"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { useMe } from "@/features/auth";
import {
  useContacts,
  useConversations,
  useMessagesRealtime,
} from "@/features/messages";
import { ConversationList } from "@/features/messages/components/ConversationList";
import { ContactList } from "@/features/messages/components/ContactList";
import { Button } from "@/shared/ui/Button";
import { Dialog } from "@/shared/ui/Dialog";

export default function MessagesPage() {
  const t = useTranslations("messages");
  const commonT = useTranslations("common");
  const { data: user } = useMe();
  const { data: conversations, isLoading: conversationsLoading } =
    useConversations();
  const { contacts, isLoading: contactsLoading } = useContacts(user?.id);
  const [newMessageOpen, setNewMessageOpen] = useState(false);

  useMessagesRealtime(user?.username);

  const conversationPartnerIds = new Set(
    (conversations ?? []).map((message) =>
      message.sender_id === user?.id ? message.recipient_id : message.sender_id,
    ),
  );
  const newContacts = contacts.filter(
    (contact) => !conversationPartnerIds.has(contact.user_id),
  );

  return (
    <div className="flex flex-col gap-6 max-w-md">
      <div className="flex items-center justify-between">
        <h1 className="text-title font-medium text-[var(--color-text-primary)]">
          {t("heading")}
        </h1>
        <Button variant="secondary" onClick={() => setNewMessageOpen(true)}>
          {t("newMessage")}
        </Button>
      </div>

      {conversationsLoading ? (
        <p className="text-body text-[var(--color-text-muted)]">
          {commonT("loading")}
        </p>
      ) : (
        <ConversationList
          conversations={conversations ?? []}
          contacts={contacts}
          currentUserId={user?.id ?? ""}
        />
      )}

      <Dialog
        isOpen={newMessageOpen}
        onClose={() => setNewMessageOpen(false)}
        title={t("newMessage")}
      >
        {contactsLoading ? (
          <p className="text-body text-[var(--color-text-muted)]">
            {commonT("loading")}
          </p>
        ) : (
          <ContactList
            contacts={newContacts}
            emptyMessage={
              contacts.length > 0 ? t("alreadyMessagedEveryone") : undefined
            }
          />
        )}
      </Dialog>
    </div>
  );
}
