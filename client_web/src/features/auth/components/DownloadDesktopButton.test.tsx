import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { NextIntlClientProvider } from "next-intl";
import { DownloadDesktopButton } from "./DownloadDesktopButton";
import messages from "../../../../messages/en.json";

describe("DownloadDesktopButton", () => {
  it("links to the desktop binary download route", () => {
    render(
      <NextIntlClientProvider locale="en" messages={messages}>
        <DownloadDesktopButton />
      </NextIntlClientProvider>,
    );

    expect(
      screen.getByRole("link", { name: "Download desktop app" }),
    ).toHaveAttribute("href", "/client.dmg");
  });
});
