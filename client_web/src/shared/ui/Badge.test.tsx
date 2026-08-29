import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { Flame } from "lucide-react";
import { Badge } from "./Badge";

describe("Badge", () => {
  it("renders the label", () => {
    render(<Badge label="Critical" icon={Flame} color="danger" />);
    expect(screen.getByText("Critical")).toBeInTheDocument();
  });

  it("applies the color's class for each variant", () => {
    const { rerender } = render(
      <Badge label="Test" icon={Flame} color="danger" />,
    );
    expect(screen.getByText("Test").className).toContain(
      "text-[var(--color-danger)]",
    );

    rerender(<Badge label="Test" icon={Flame} color="success" />);
    expect(screen.getByText("Test").className).toContain(
      "text-[var(--color-success)]",
    );

    rerender(<Badge label="Test" icon={Flame} color="neutral" />);
    expect(screen.getByText("Test").className).toContain(
      "text-[var(--color-text-secondary)]",
    );
  });
});
