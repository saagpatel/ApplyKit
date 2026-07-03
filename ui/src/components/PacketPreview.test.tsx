import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { PacketDetail } from "../lib/types";
import { PacketPreview } from "./PacketPreview";

function detail(): PacketDetail {
  return {
    packetDir: "/tmp/applykit_packets/Acme_Senior_Engineer_2026-07-03",
    extractionSource: "deterministic",
    extractedKeywords: ["automation", "workflow"],
    extractedTools: ["Rust"],
    extractedRequirements: ["Build tooling"],
    fitBreakdown: {
      roleMatch: 10,
      stackMatch: 5,
      scaleMatch: 0,
      rigorMatch: 5,
      signalBoost: 5,
      total: 25,
      whyMatch: ["Primary track aligned"],
      gaps: ["Missing scale signal"]
    },
    track: "automation_ai_ops",
    trackScores: [["automation_ai_ops", 20, ["automation"]]],
    tailorPlan: {
      maxResumeEdits: 3,
      maxBulletSwaps: 2,
      edits: [
        {
          kind: "rewrite",
          targetSection: "Experience",
          reason: "Align to deterministic automation",
          provenanceIds: ["bullet-1"]
        }
      ]
    },
    bulletCandidates: [],
    messages: {
      recruiter: "Recruiter message",
      hiringManager: "Hiring manager message",
      coverShort: "Cover short"
    },
    resume1pg: "# Resume\n- Built reliable tooling",
    diff: "  # Resume\n- Old bullet\n+ Built reliable tooling",
    trackerRow: {
      date: "2026-07-03",
      company: "Acme",
      role: "Senior Engineer",
      source: "LinkedIn",
      track: "automation_ai_ops",
      fitTotal: 25,
      status: "new",
      nextAction: "follow up",
      packetDir: "/tmp/applykit_packets/Acme_Senior_Engineer_2026-07-03"
    },
    truthReport: {
      passed: true,
      violations: [],
      unknownTools: [],
      claimIssues: [],
      provenanceComplete: true
    }
  };
}

describe("PacketPreview", () => {
  it("switches between generated packet artifacts", () => {
    render(<PacketPreview detail={detail()} />);

    expect(screen.getByRole("heading", { name: "Packet Preview" })).toBeInTheDocument();
    expect(screen.getByText("FitScore.md")).toBeInTheDocument();
    expect(screen.getByText("25 / 100")).toBeInTheDocument();

    const artifactTabs = screen.getByRole("tablist", { name: "Packet artifacts" });
    fireEvent.click(within(artifactTabs).getByRole("tab", { name: "Plan" }));
    expect(screen.getByText("TailorPlan.md")).toBeInTheDocument();
    expect(screen.getAllByText((_, node) => node?.textContent === "rewrite [Experience] Align to deterministic automation")).toHaveLength(2);

    fireEvent.click(within(artifactTabs).getByRole("tab", { name: "Resume" }));
    expect(screen.getByText("Resume_1pg_Tailored.md")).toBeInTheDocument();
    expect(screen.getByText("Built reliable tooling")).toBeInTheDocument();

    fireEvent.click(within(artifactTabs).getByRole("tab", { name: "Diff" }));
    expect(screen.getByText("Diff.md")).toBeInTheDocument();
    expect(screen.getByText("Diff Viewer")).toBeInTheDocument();
  });

  it("renders an empty state without a packet", () => {
    render(<PacketPreview />);

    expect(screen.getByText("No packet selected yet.")).toBeInTheDocument();
  });
});
