import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { Button } from "./Button";

describe("Button", () => {
  it("renders children", () => {
    render(<Button>Create incident</Button>);
    expect(
      screen.getByRole("button", { name: "Create incident" }),
    ).toBeInTheDocument();
  });

  it("renders primary variant", () => {
    render(<Button variant="primary">Submit</Button>);
    const btn = screen.getByRole("button");
    expect(btn.className).toContain("bg-[var(--color-accent)]");
  });

  it("renders secondary variant", () => {
    render(<Button variant="secondary">Cancel</Button>);
    const btn = screen.getByRole("button");
    expect(btn.className).toContain("bg-transparent");
  });

  it("renders danger variant", () => {
    render(<Button variant="danger">Delete</Button>);
    const btn = screen.getByRole("button");
    expect(btn.className).toContain("text-[var(--color-danger)]");
  });

  it("is disabled when isLoading is true", () => {
    render(<Button isLoading>Loading</Button>);
    expect(screen.getByRole("button")).toBeDisabled();
  });

  it("is disabled when disabled prop is true", () => {
    render(<Button disabled>Disabled</Button>);
    expect(screen.getByRole("button")).toBeDisabled();
  });

  it("shows spinner when isLoading is true", () => {
    render(<Button isLoading>Loading</Button>);
    const spinner = screen.getByRole("button").querySelector(".animate-spin");
    expect(spinner).toBeInTheDocument();
  });

  it("renders as a link when href is set", () => {
    render(<Button href="/somewhere">Go</Button>);
    const link = screen.getByRole("link", { name: "Go" });
    expect(link).toHaveAttribute("href", "/somewhere");
  });

  it("applies the variant styling when rendered as a link", () => {
    render(
      <Button href="/somewhere" variant="primary">
        Go
      </Button>,
    );
    const link = screen.getByRole("link", { name: "Go" });
    expect(link.className).toContain("bg-[var(--color-accent)]");
  });
});
