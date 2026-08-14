import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Dialog } from "./Dialog";

beforeEach(() => {
  HTMLDialogElement.prototype.showModal = vi.fn();
  HTMLDialogElement.prototype.close = vi.fn();
});

const renderDialog = (isOpen: boolean, onClose: () => void = () => {}) =>
  render(
    <Dialog isOpen={isOpen} onClose={onClose} title="Create team">
      <p>Content</p>
    </Dialog>,
  );

describe("Dialog", () => {
  it("renders nothing when closed", () => {
    renderDialog(false);
    expect(screen.queryByText("Create team")).not.toBeInTheDocument();
  });

  it("renders title and children when open", () => {
    renderDialog(true);
    expect(screen.getByText("Create team")).toBeInTheDocument();
    expect(screen.getByText("Content")).toBeInTheDocument();
  });

  it("calls onClose when close button is clicked", async () => {
    const onClose = vi.fn();
    renderDialog(true, onClose);
    await userEvent.click(
      screen.getByRole("button", { name: "Close dialog", hidden: true }),
    );
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("calls close when dialog transitions from open to false", () => {
    const { rerender } = renderDialog(true);

    rerender(
      <Dialog isOpen={false} onClose={() => {}} title="Create team">
        <p>Content</p>
      </Dialog>,
    );

    expect(HTMLDialogElement.prototype.close).toHaveBeenCalled();
  });

  it("calls onClose when the dialog fires a native cancel event (Escape key)", () => {
    const onClose = vi.fn();
    renderDialog(true, onClose);

    const dialog = screen.getByRole("dialog", { hidden: true });
    const cancelEvent = new Event("cancel", { cancelable: true });
    dialog.dispatchEvent(cancelEvent);

    expect(onClose).toHaveBeenCalledOnce();
    expect(cancelEvent.defaultPrevented).toBe(true);
  });

  it("calls onClose when clicking the backdrop (outside the dialog box)", async () => {
    const onClose = vi.fn();
    renderDialog(true, onClose);

    await userEvent.click(screen.getByRole("dialog", { hidden: true }));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("does not call onClose when clicking inside the dialog content", async () => {
    const onClose = vi.fn();
    renderDialog(true, onClose);

    await userEvent.click(screen.getByText("Content"));
    expect(onClose).not.toHaveBeenCalled();
  });
});
