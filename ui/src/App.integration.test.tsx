import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import { invokeSafe } from "./lib/tauri";
import type { GenerateResponse, JobSummary, PacketDetail, SettingsModel } from "./lib/types";

vi.mock("./lib/tauri", () => ({
  invokeSafe: vi.fn()
}));

const defaultSettings: SettingsModel = {
  allowUnapproved: false,
  llmEnabled: true,
  llmProvider: "ollama",
  llmBaseUrl: "http://127.0.0.1:11434",
  llmModel: "llama3.2",
  llmAllowedTasks: ["rewrite_message", "rewrite_bullet", "summarize_jd"]
};

function packetDetail(packetDir: string, status = "new", nextAction = "follow up"): PacketDetail {
  return {
    packetDir,
    extractionSource: "deterministic",
    extractionDiagnostics: {
      summarizeAttempted: false,
      summarizeMerged: false,
      summarizeFallbackReasons: []
    },
    extractedKeywords: ["support"],
    extractedTools: ["Okta"],
    extractedRequirements: ["Experience with incidents"],
    fitBreakdown: {
      roleMatch: 25,
      stackMatch: 10,
      scaleMatch: 5,
      rigorMatch: 5,
      signalBoost: 5,
      total: 50,
      whyMatch: ["Primary track aligned: Support/Ops Core"],
      gaps: []
    },
    track: "Support/Ops Core",
    trackScores: [["Support/Ops Core", 30, ["support"]]],
    tailorPlan: {
      maxResumeEdits: 3,
      maxBulletSwaps: 2,
      edits: []
    },
    bulletCandidates: [],
    messages: {
      recruiter: "Recruiter message",
      hiringManager: "Hiring manager message",
      coverShort: "Cover short"
    },
    resume1pg: "# Resume\n- Bullet",
    diff: "# Diff",
    trackerRow: {
      date: "2026-03-14",
      company: "Acme",
      role: "Senior Support Engineer",
      source: "LinkedIn",
      track: "Support/Ops Core",
      fitTotal: 50,
      status,
      nextAction,
      packetDir
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

function generateResponse(packetDir: string): GenerateResponse {
  return {
    packetDir,
    fitTotal: 50,
    track: "Support/Ops Core",
    filesWritten: [],
    truthPassed: true,
    packetDetail: packetDetail(packetDir)
  };
}

function generatedJob(packetDir: string): JobSummary {
  return {
    id: "job-1",
    company: "Acme",
    role: "Senior Support Engineer",
    source: "LinkedIn",
    baseline: "1pg",
    track: "Support/Ops Core",
    fitTotal: 50,
    status: "new",
    nextAction: "follow up",
    outputDir: packetDir,
    updatedAt: "2026-03-14T12:00:00Z"
  };
}

function installClipboardMock(writeText: ReturnType<typeof vi.fn>) {
  Object.defineProperty(window.navigator, "clipboard", {
    value: { writeText },
    configurable: true
  });
}

async function generatePacketInApp() {
  fireEvent.click(screen.getAllByRole("button", { name: "New Job" })[0]);
  fireEvent.change(screen.getByPlaceholderText("Acme"), { target: { value: "Acme" } });
  fireEvent.change(screen.getByPlaceholderText("Senior Support Engineer"), {
    target: { value: "Senior Support Engineer" }
  });
  fireEvent.change(screen.getByPlaceholderText("LinkedIn"), { target: { value: "LinkedIn" } });
  fireEvent.change(screen.getByPlaceholderText("Paste job description here"), {
    target: { value: "Support engineer JD" }
  });
  fireEvent.click(screen.getByRole("button", { name: "Generate Packet (⌘Enter)" }));

  await screen.findByRole("heading", { name: "Job Review" });
}

describe("App workflow integration", () => {
  const invokeSafeMock = vi.mocked(invokeSafe);

  beforeEach(() => {
    invokeSafeMock.mockReset();
  });

  afterEach(() => {
    cleanup();
    vi.useRealTimers();
  });

  it("defers startup data loads until after the initial render", async () => {
    invokeSafeMock.mockResolvedValue({});

    render(<App />);

    expect(screen.getByText("ApplyKit Dashboard")).toBeInTheDocument();
    expect(invokeSafeMock).not.toHaveBeenCalled();

    await waitFor(() => {
      expect(invokeSafeMock).toHaveBeenCalledWith("list_jobs_cmd", {});
      expect(invokeSafeMock).toHaveBeenCalledWith("insights_cmd", {});
      expect(invokeSafeMock).toHaveBeenCalledWith("get_settings_cmd", {});
    });
  });

  it("shows a retryable error instead of a genuine empty state when saved jobs fail to load", async () => {
    invokeSafeMock.mockImplementation(async (command) => {
      if (command === "list_jobs_cmd") {
        throw new Error("database unavailable");
      }
      if (command === "insights_cmd") {
        return { repliesByTrack: [], commonGaps: [], keywordCorrelations: [] };
      }
      if (command === "get_settings_cmd") {
        return defaultSettings;
      }
      throw new Error(`Unexpected command: ${command}`);
    });

    render(<App />);

    expect(
      await screen.findByText("Saved jobs could not be loaded")
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Retry" })).toBeInTheDocument();
    expect(screen.queryByText("No jobs yet. Generate your first packet.")).not.toBeInTheDocument();
  });

  it("resolves the generated job id before saving tracker updates", async () => {
    const packetDir = "/tmp/applykit_packets/Acme_Senior_Support_Engineer_2026-03-14";
    let listJobsCalls = 0;

    invokeSafeMock.mockImplementation(async (command, payload) => {
      switch (command) {
        case "list_jobs_cmd":
          listJobsCalls += 1;
          return { jobs: listJobsCalls === 1 ? [] : [generatedJob(packetDir)] };
        case "insights_cmd":
          return { repliesByTrack: [], commonGaps: [], keywordCorrelations: [] };
        case "get_settings_cmd":
          return defaultSettings;
        case "generate_packet_cmd":
          return generateResponse(packetDir);
        case "update_job_status_cmd":
          expect(payload).toEqual({
            input: {
              id: "job-1",
              status: "reply",
              nextAction: "send follow-up",
              notes: "prep examples"
            }
          });
          return {
            ok: true,
            id: "job-1",
            status: "reply",
            nextAction: "send follow-up",
            notes: "prep examples"
          };
        case "get_packet_detail_cmd":
          expect(payload).toEqual({ input: { jobId: "job-1" } });
          return packetDetail(packetDir, "reply", "send follow-up");
        default:
          throw new Error(`Unexpected command: ${command}`);
      }
    });

    render(<App />);
    await generatePacketInApp();

    fireEvent.click(screen.getByRole("tab", { name: /tracker/i }));
    fireEvent.change(screen.getByDisplayValue("new"), { target: { value: "reply" } });
    fireEvent.change(screen.getByPlaceholderText("Follow up on Monday"), {
      target: { value: "send follow-up" }
    });
    fireEvent.change(screen.getByPlaceholderText("Interview prep notes"), {
      target: { value: "prep examples" }
    });
    fireEvent.click(screen.getByRole("button", { name: "Save Tracker" }));

    await waitFor(() => {
      expect(invokeSafeMock).toHaveBeenCalledWith("update_job_status_cmd", {
        input: {
          id: "job-1",
          status: "reply",
          nextAction: "send follow-up",
          notes: "prep examples"
        }
      });
    });
    expect(screen.queryByText("Could not resolve job id for tracker update")).not.toBeInTheDocument();
  });

  it("shows an error toast when an export action fails", async () => {
    const packetDir = "/tmp/applykit_packets/Acme_Senior_Support_Engineer_2026-03-14";

    invokeSafeMock.mockImplementation(async (command) => {
      switch (command) {
        case "list_jobs_cmd":
          return { jobs: [] };
        case "insights_cmd":
          return { repliesByTrack: [], commonGaps: [], keywordCorrelations: [] };
        case "get_settings_cmd":
          return defaultSettings;
        case "generate_packet_cmd":
          return generateResponse(packetDir);
        case "export_pdf_cmd":
          throw new Error("PDF export failed");
        default:
          throw new Error(`Unexpected command: ${command}`);
      }
    });

    render(<App />);
    await generatePacketInApp();

    fireEvent.click(screen.getByRole("tab", { name: /export/i }));
    fireEvent.click(screen.getByRole("button", { name: "Export PDF" }));

    expect(await screen.findByText("PDF export failed")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Job Review" })).toBeInTheDocument();
  });

  it("supports message copy, open folder, and exports from job review", async () => {
    const packetDir = "/tmp/applykit_packets/Acme_Senior_Support_Engineer_2026-03-14";
    const writeText = vi.fn().mockResolvedValue(undefined);
    installClipboardMock(writeText);

    invokeSafeMock.mockImplementation(async (command, payload) => {
      switch (command) {
        case "list_jobs_cmd":
          return { jobs: [] };
        case "insights_cmd":
          return { repliesByTrack: [], commonGaps: [], keywordCorrelations: [] };
        case "get_settings_cmd":
          return defaultSettings;
        case "generate_packet_cmd":
          return generateResponse(packetDir);
        case "open_output_folder":
          expect(payload).toEqual({ path: packetDir });
          return undefined;
        case "export_markdown_cmd":
          expect(payload).toEqual({ input: { packetDir } });
          return { ok: true, message: "Markdown export complete" };
        case "export_docx_cmd":
          expect(payload).toEqual({ input: { packetDir } });
          return { ok: true, message: "DOCX export complete" };
        case "export_pdf_cmd":
          expect(payload).toEqual({ input: { packetDir } });
          return { ok: true, message: "PDF export complete" };
        default:
          throw new Error(`Unexpected command: ${command}`);
      }
    });

    render(<App />);
    await generatePacketInApp();

    fireEvent.click(screen.getByRole("button", { name: "Open Folder" }));
    await waitFor(() => {
      expect(invokeSafeMock).toHaveBeenCalledWith("open_output_folder", { path: packetDir });
    });

    fireEvent.click(screen.getByRole("tab", { name: /messages/i }));
    fireEvent.click(screen.getAllByRole("button", { name: "Copy" })[0]);
    expect(writeText).toHaveBeenCalledWith("Recruiter message");
    expect(await screen.findByText("Copied")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("tab", { name: /export/i }));
    fireEvent.click(screen.getByRole("button", { name: "Export Markdown Bundle" }));
    fireEvent.click(screen.getByRole("button", { name: "Export DOCX" }));
    fireEvent.click(screen.getByRole("button", { name: "Export PDF" }));

    await screen.findByText("Markdown export complete");
    expect(screen.getByText("DOCX export complete")).toBeInTheDocument();
    expect(screen.getByText("PDF export complete")).toBeInTheDocument();
  });

  it("shows generated packet artifacts in the preview pane", async () => {
    const packetDir = "/tmp/applykit_packets/Acme_Senior_Support_Engineer_2026-03-14";

    invokeSafeMock.mockImplementation(async (command) => {
      switch (command) {
        case "list_jobs_cmd":
          return { jobs: [] };
        case "insights_cmd":
          return { repliesByTrack: [], commonGaps: [], keywordCorrelations: [] };
        case "get_settings_cmd":
          return defaultSettings;
        case "generate_packet_cmd":
          return generateResponse(packetDir);
        default:
          throw new Error(`Unexpected command: ${command}`);
      }
    });

    render(<App />);
    await generatePacketInApp();

    expect(screen.getByRole("heading", { name: "Packet Preview" })).toBeInTheDocument();
    expect(screen.getByText("FitScore.md")).toBeInTheDocument();
    expect(screen.getByText("50 / 100")).toBeInTheDocument();

    const artifactTabs = screen.getByRole("tablist", { name: "Packet artifacts" });
    fireEvent.click(within(artifactTabs).getByRole("tab", { name: "Plan" }));
    expect(screen.getByText("TailorPlan.md")).toBeInTheDocument();

    fireEvent.click(within(artifactTabs).getByRole("tab", { name: "Resume" }));
    expect(screen.getByText("Resume_1pg_Tailored.md")).toBeInTheDocument();
  });

  it("handles the packet-path copy shortcut for both success and clipboard errors", async () => {
    const packetDir = "/tmp/applykit_packets/Acme_Senior_Support_Engineer_2026-03-14";
    const writeText = vi
      .fn()
      .mockResolvedValueOnce(undefined)
      .mockRejectedValueOnce(new Error("Clipboard blocked"));
    installClipboardMock(writeText);

    invokeSafeMock.mockImplementation(async (command) => {
      switch (command) {
        case "list_jobs_cmd":
          return { jobs: [] };
        case "insights_cmd":
          return { repliesByTrack: [], commonGaps: [], keywordCorrelations: [] };
        case "get_settings_cmd":
          return defaultSettings;
        case "generate_packet_cmd":
          return generateResponse(packetDir);
        default:
          throw new Error(`Unexpected command: ${command}`);
      }
    });

    render(<App />);
    await generatePacketInApp();

    fireEvent.keyDown(window, { key: "C", ctrlKey: true, shiftKey: true });
    expect(await screen.findByText("Copied packet path")).toBeInTheDocument();

    fireEvent.keyDown(window, { key: "C", ctrlKey: true, shiftKey: true });
    expect(await screen.findByText("Clipboard blocked")).toBeInTheDocument();
  });
});
