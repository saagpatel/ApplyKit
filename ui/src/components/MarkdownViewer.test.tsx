import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { MarkdownViewer } from "./MarkdownViewer";

describe("MarkdownViewer", () => {
  it("renders packet markdown structure without exposing hidden comments", () => {
    const { container } = render(
      <MarkdownViewer
        markdown={[
          "<!-- APPLYKIT_RESUME_TEMPLATE v1 -->",
          "# Packet",
          "",
          "Total: **50 / 100**",
          "",
          "## Steps",
          "1. **rewrite** [Experience] Align to job",
          "2. Keep `Truth Gate` grounded",
          "",
          "- Bullet one",
          "- Bullet two",
          "",
          "> Verified locally",
          "",
          "```",
          "raw packet text",
          "```"
        ].join("\n")}
      />
    );

    expect(screen.getByRole("heading", { name: "Packet" })).toBeInTheDocument();
    expect(screen.getByText("50 / 100")).toBeInTheDocument();
    expect(screen.getByText("Truth Gate")).toBeInTheDocument();
    expect(screen.getByText("Verified locally")).toBeInTheDocument();
    expect(screen.getByText("raw packet text")).toBeInTheDocument();
    expect(container).not.toHaveTextContent("APPLYKIT_RESUME_TEMPLATE");

    const lists = container.querySelectorAll("ol, ul");
    expect(lists).toHaveLength(2);
    expect(within(lists[0] as HTMLElement).getByText("rewrite")).toBeInTheDocument();
    expect(within(lists[1] as HTMLElement).getByText("Bullet one")).toBeInTheDocument();
  });
});
