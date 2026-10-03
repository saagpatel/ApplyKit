import { useState } from "react";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { CommandPalette } from "./CommandPalette";

afterEach(() => {
  cleanup();
});

describe("CommandPalette", () => {
  it("navigates selected command and closes palette", () => {
    const onNavigate = vi.fn();
    const onOpenChange = vi.fn();

    render(<CommandPalette open onOpenChange={onOpenChange} onNavigate={onNavigate} />);

    fireEvent.click(screen.getByText("Go to Dashboard"));

    expect(onNavigate).toHaveBeenCalledWith("dashboard");
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it("preserves fuzzy value ranking and arrow/Enter selection", () => {
    const onNavigate = vi.fn();
    render(<CommandPalette open onOpenChange={vi.fn()} onNavigate={onNavigate} />);
    const input = screen.getByRole("combobox");
    fireEvent.change(input, { target: { value: "job" } });
    expect(screen.getAllByRole("option").map((item) => item.textContent)).toEqual(["Go to Job Review", "Go to New Job"]);
    fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(screen.getByRole("option", { name: "Go to New Job" })).toHaveAttribute("aria-selected", "true");
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onNavigate).toHaveBeenCalledWith("new-job");
  });

  it("keeps boundary keys, pointer selection, and no-results behavior", () => {
    const onNavigate = vi.fn();
    render(<CommandPalette open onOpenChange={vi.fn()} onNavigate={onNavigate} />);
    const input = screen.getByRole("combobox");
    const selected = () => screen.getAllByRole("option").find((item) => item.getAttribute("aria-selected") === "true");
    fireEvent.keyDown(input, { key: "ArrowUp" });
    expect(selected()).toHaveTextContent("Go to Dashboard");
    fireEvent.keyDown(input, { key: "End" });
    fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(selected()).toHaveTextContent("Go to Settings");
    fireEvent.keyDown(input, { key: "Home" });
    fireEvent.keyDown(input, { key: "n", ctrlKey: true });
    expect(selected()).toHaveTextContent("Go to New Job");
    fireEvent.keyDown(input, { key: "p", ctrlKey: true });
    expect(selected()).toHaveTextContent("Go to Dashboard");
    fireEvent.keyDown(input, { key: "ArrowDown", metaKey: true });
    expect(selected()).toHaveTextContent("Go to Settings");
    fireEvent.keyDown(input, { key: "ArrowUp", metaKey: true });
    expect(selected()).toHaveTextContent("Go to Dashboard");
    fireEvent.pointerMove(screen.getByRole("option", { name: "Go to Banks" }));
    expect(selected()).toHaveTextContent("Go to Banks");
    expect(fireEvent.keyDown(input, { key: "Tab" })).toBe(false);
    expect(input).toHaveFocus();
    expect(fireEvent.keyDown(input, { key: "Tab", shiftKey: true })).toBe(false);
    expect(input).toHaveFocus();
    fireEvent.change(input, { target: { value: "not-a-command" } });
    expect(screen.getByRole("status")).toHaveTextContent("No results.");
    expect(input).not.toHaveAttribute("aria-activedescendant");
    fireEvent.keyDown(input, { key: "Enter" });
    fireEvent.keyDown(input, { key: "End" });
    expect(onNavigate).not.toHaveBeenCalled();
  });

  it("does not select commands during IME composition", () => {
    const onNavigate = vi.fn();
    render(<CommandPalette open onOpenChange={vi.fn()} onNavigate={onNavigate} />);
    const input = screen.getByRole("combobox");
    fireEvent.keyDown(input, { key: "Enter", isComposing: true });
    fireEvent.keyDown(input, { key: "Enter", keyCode: 229 });
    expect(onNavigate).not.toHaveBeenCalled();
  });

  it("locks background scroll while open and restores existing inline styles", () => {
    document.documentElement.style.setProperty("overflow-x", "auto", "important");
    document.documentElement.style.overflowY = "scroll";
    document.body.style.overflowX = "clip";
    document.body.style.setProperty("overflow-y", "auto", "important");
    document.body.style.paddingRight = "7px";
    const { unmount } = render(<CommandPalette open onOpenChange={vi.fn()} onNavigate={vi.fn()} />);
    expect(document.documentElement.style.overflow).toBe("hidden");
    expect(document.body.style.overflow).toBe("hidden");
    expect(screen.getByRole("combobox")).toHaveAttribute("autocomplete", "off");
    expect(screen.getByRole("combobox")).toHaveAttribute("autocorrect", "off");
    expect(screen.getByRole("combobox")).toHaveAttribute("spellcheck", "false");
    unmount();
    expect(document.documentElement.style.overflowX).toBe("auto");
    expect(document.documentElement.style.getPropertyPriority("overflow-x")).toBe("important");
    expect(document.documentElement.style.overflowY).toBe("scroll");
    expect(document.body.style.overflowX).toBe("clip");
    expect(document.body.style.overflowY).toBe("auto");
    expect(document.body.style.getPropertyPriority("overflow-y")).toBe("important");
    expect(document.body.style.paddingRight).toBe("7px");
    for (const element of [document.documentElement, document.body]) {
      element.style.removeProperty("overflow-x");
      element.style.removeProperty("overflow-y");
    }
    document.body.style.removeProperty("padding-right");
  });

  it("restores focus and clears search on close", async () => {
    function Harness() {
      const [open, setOpen] = useState(false);
      return (
        <>
          <button type="button" onClick={() => setOpen(true)}>
            Open palette
          </button>
          <CommandPalette open={open} onOpenChange={setOpen} onNavigate={vi.fn()} />
        </>
      );
    }

    render(<Harness />);

    const opener = screen.getByRole("button", { name: "Open palette" });
    opener.focus();
    fireEvent.click(opener);

    const dialog = screen.getByRole("dialog", { name: /command palette/i });
    const input = within(dialog).getByPlaceholderText("Type a command...");
    fireEvent.change(input, { target: { value: "Dashboard" } });
    fireEvent(dialog, new Event("cancel", { cancelable: true }));

    await waitFor(() => {
      expect(screen.queryByRole("dialog", { name: /command palette/i })).not.toBeInTheDocument();
      expect(opener).toHaveFocus();
    });
    fireEvent.click(opener);
    expect(screen.getByRole("combobox")).toHaveValue("");
    expect(screen.getAllByRole("option")).toHaveLength(6);
  });
});
