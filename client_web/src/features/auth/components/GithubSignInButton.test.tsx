import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { NextIntlClientProvider } from "next-intl";
import { GithubSignInButton } from "./GithubSignInButton";
import { githubSignInUrl } from "../api";
import messages from "../../../../messages/en.json";

describe("GithubSignInButton", () => {
  it("links to the GitHub OAuth sign-in URL", () => {
    render(
      <NextIntlClientProvider locale="en" messages={messages}>
        <GithubSignInButton />
      </NextIntlClientProvider>,
    );

    expect(
      screen.getByRole("link", { name: "Continue with GitHub" }),
    ).toHaveAttribute("href", githubSignInUrl);
  });
});
