import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { NextIntlClientProvider } from "next-intl";
import { ContactList } from "./ContactList";
import { Contact } from "../hooks";
import messages from "../../../../messages/en.json";

const contact: Contact = { user_id: "user-456", username: "bob" };

const renderList = (contacts: Contact[], emptyMessage?: string) =>
  render(
    <NextIntlClientProvider locale="en" messages={messages}>
      <ContactList contacts={contacts} emptyMessage={emptyMessage} />
    </NextIntlClientProvider>,
  );

describe("ContactList", () => {
  it("shows the default empty state message when there are no contacts", () => {
    renderList([]);
    expect(
      screen.getByText("No teammates to message yet. Join a team first."),
    ).toBeInTheDocument();
  });

  it("shows a custom empty state message when provided", () => {
    renderList([], "Nobody here");
    expect(screen.getByText("Nobody here")).toBeInTheDocument();
  });

  it("renders each contact's username", () => {
    renderList([contact]);
    expect(screen.getByText("bob")).toBeInTheDocument();
  });

  it("links each contact to their conversation", () => {
    renderList([contact]);
    expect(screen.getByRole("link")).toHaveAttribute(
      "href",
      "/messages/user-456",
    );
  });
});
