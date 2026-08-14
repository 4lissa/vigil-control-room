import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Dropdown } from "./Dropdown";

const renderDropdown = () =>
  render(
    <Dropdown
      trigger={({ onClick }) => <button onClick={onClick}>Open</button>}
    >
      {(close) => <button onClick={close}>Item</button>}
    </Dropdown>,
  );

describe("Dropdown", () => {
  it("does not render the panel until the trigger is clicked", () => {
    renderDropdown();
    expect(screen.queryByText("Item")).not.toBeInTheDocument();
  });

  it("opens the panel when the trigger is clicked", async () => {
    renderDropdown();
    await userEvent.click(screen.getByText("Open"));
    expect(screen.getByText("Item")).toBeInTheDocument();
  });

  it("toggles closed when the trigger is clicked again", async () => {
    renderDropdown();
    await userEvent.click(screen.getByText("Open"));
    await userEvent.click(screen.getByText("Open"));
    expect(screen.queryByText("Item")).not.toBeInTheDocument();
  });

  it("closes when clicking outside the dropdown", async () => {
    render(
      <div>
        <Dropdown
          trigger={({ onClick }) => <button onClick={onClick}>Open</button>}
        >
          {() => <span>Item</span>}
        </Dropdown>
        <button>Outside</button>
      </div>,
    );
    await userEvent.click(screen.getByText("Open"));
    expect(screen.getByText("Item")).toBeInTheDocument();

    await userEvent.click(screen.getByText("Outside"));
    expect(screen.queryByText("Item")).not.toBeInTheDocument();
  });

  it("calls the close callback passed to children", async () => {
    const onClose = vi.fn();
    render(
      <Dropdown
        trigger={({ onClick }) => <button onClick={onClick}>Open</button>}
      >
        {(close) => (
          <button
            onClick={() => {
              onClose();
              close();
            }}
          >
            Item
          </button>
        )}
      </Dropdown>,
    );
    await userEvent.click(screen.getByText("Open"));
    await userEvent.click(screen.getByText("Item"));
    expect(onClose).toHaveBeenCalledOnce();
    expect(screen.queryByText("Item")).not.toBeInTheDocument();
  });
});
