import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { NextIntlClientProvider } from "next-intl";
import { ConversationList } from "./ConversationList";
import { Contact } from "../hooks";
import { MessageResponse } from "../types";
import messages from "../../../../messages/en.json";

const contact: Contact = { user_id: "user-456", username: "bob" };

const messageFromMe: MessageResponse = {
  id: "message-1",
  sender_id: "user-123",
  recipient_id: "user-456",
  content: "Hey there",
  created_at: 1755000000,
};

const messageFromThem: MessageResponse = {
  id: "message-2",
  sender_id: "user-456",
  recipient_id: "user-123",
  content: "Hi!",
  created_at: 1755000100,
};

const renderList = (conversations: MessageResponse[]) =>
  render(
    <NextIntlClientProvider locale="en" messages={messages}>
      <ConversationList
        conversations={conversations}
        contacts={[contact]}
        currentUserId="user-123"
      />
    </NextIntlClientProvider>,
  );

describe("ConversationList", () => {
  it("shows an empty state message when there are no conversations", () => {
    renderList([]);
    expect(
      screen.getByText("No conversations yet. Start one below."),
    ).toBeInTheDocument();
  });

  it("resolves the counterpart's name whether I sent or received the last message", () => {
    renderList([messageFromMe]);
    expect(screen.getByText("bob")).toBeInTheDocument();
    expect(screen.getByText("Hey there")).toBeInTheDocument();

    renderList([messageFromThem]);
    expect(screen.getAllByText("bob")).toHaveLength(2);
    expect(screen.getByText("Hi!")).toBeInTheDocument();
  });

  it("falls back to the raw user id when the contact is unknown", () => {
    renderList([{ ...messageFromMe, recipient_id: "user-999" }]);
    expect(screen.getByText("user-999")).toBeInTheDocument();
  });

  it("links each conversation to the counterpart's thread", () => {
    renderList([messageFromMe]);
    expect(screen.getByRole("link")).toHaveAttribute(
      "href",
      "/messages/user-456",
    );
  });
});
