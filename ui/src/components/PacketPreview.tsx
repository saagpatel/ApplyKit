import { useMemo, useState } from "react";
import type { PacketDetail } from "../lib/types";
import { DiffViewer } from "./DiffViewer";
import { MarkdownViewer } from "./MarkdownViewer";

interface Props {
  detail?: PacketDetail;
}

type ArtifactKind = "markdown" | "diff";

interface Artifact {
  id: string;
  label: string;
  fileName: string;
  kind: ArtifactKind;
  markdown?: string;
  diff?: string;
}

function formatTrack(track: string) {
  return track
    .split(/[_\s/-]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function fitScoreMarkdown(detail: PacketDetail) {
  const fit = detail.fitBreakdown;
  const why = fit.whyMatch.length ? fit.whyMatch : ["No positive match reasons recorded."];
  const gaps = fit.gaps.length ? fit.gaps : ["No major gaps recorded."];

  return [
    "# Fit Score",
    "",
    `Total: **${fit.total} / 100**`,
    "",
    "## Breakdown",
    `- Role match: ${fit.roleMatch}`,
    `- Stack match: ${fit.stackMatch}`,
    `- Scale match: ${fit.scaleMatch}`,
    `- Rigor match: ${fit.rigorMatch}`,
    `- Signal boost: ${fit.signalBoost}`,
    "",
    "## Why You Match",
    ...why.map((item) => `- ${item}`),
    "",
    "## Gaps",
    ...gaps.map((item) => `- ${item}`)
  ].join("\n");
}

function tailorPlanMarkdown(detail: PacketDetail) {
  const edits = detail.tailorPlan.edits.length
    ? detail.tailorPlan.edits.map(
        (edit, idx) => `${idx + 1}. **${edit.kind}** [${edit.targetSection}] ${edit.reason}`
      )
    : ["1. No resume edits proposed."];

  return [
    "# Tailor Plan",
    "",
    `Max resume edits: **${detail.tailorPlan.maxResumeEdits}**`,
    `Max bullet swaps: **${detail.tailorPlan.maxBulletSwaps}**`,
    "",
    "## Planned Edits",
    ...edits
  ].join("\n");
}

function buildArtifacts(detail: PacketDetail): Artifact[] {
  const artifacts: Artifact[] = [
    {
      id: "fit",
      label: "Fit",
      fileName: "FitScore.md",
      kind: "markdown",
      markdown: fitScoreMarkdown(detail)
    },
    {
      id: "plan",
      label: "Plan",
      fileName: "TailorPlan.md",
      kind: "markdown",
      markdown: tailorPlanMarkdown(detail)
    },
    {
      id: "resume",
      label: "Resume",
      fileName: "Resume_1pg_Tailored.md",
      kind: "markdown",
      markdown: detail.resume1pg
    },
    {
      id: "diff",
      label: "Diff",
      fileName: "Diff.md",
      kind: "diff",
      diff: detail.diff
    },
    {
      id: "recruiter",
      label: "Recruiter",
      fileName: "RecruiterMessage.md",
      kind: "markdown",
      markdown: detail.messages.recruiter
    },
    {
      id: "manager",
      label: "Manager",
      fileName: "HiringManagerMessage.md",
      kind: "markdown",
      markdown: detail.messages.hiringManager
    },
    {
      id: "cover",
      label: "Cover",
      fileName: "CoverNote_Short.md",
      kind: "markdown",
      markdown: detail.messages.coverShort
    }
  ];

  if (detail.resume2pg) {
    artifacts.splice(3, 0, {
      id: "resume-2pg",
      label: "2pg",
      fileName: "Resume_2pg_Tailored.md",
      kind: "markdown",
      markdown: detail.resume2pg
    });
  }

  return artifacts;
}

export function PacketPreview({ detail }: Props) {
  const artifacts = useMemo(() => (detail ? buildArtifacts(detail) : []), [detail]);
  const [selectedId, setSelectedId] = useState("fit");
  const selected = artifacts.find((artifact) => artifact.id === selectedId) ?? artifacts[0];

  if (!detail || !selected) {
    return (
      <div className="stack-lg">
        <section className="card">
          <h3>Packet Preview</h3>
          <p className="subtle">Generate or open a packet to preview tailored artifacts.</p>
        </section>
        <section className="card code-preview">
          <pre>No packet selected yet.</pre>
        </section>
      </div>
    );
  }

  return (
    <div className="stack-lg">
      <section className="card stack-sm">
        <div className="row between wrap">
          <div>
            <h3>Packet Preview</h3>
            <p className="subtle">{detail.packetDir}</p>
          </div>
          <span className="chip">{detail.extractionSource ?? "deterministic"}</span>
        </div>
        {detail.extractionDiagnostics?.summarizeFallbackReasons?.length ? (
          <p className="subtle">
            summarize_jd fallback: {detail.extractionDiagnostics.summarizeFallbackReasons.join(", ")}
          </p>
        ) : null}
      </section>

      <section className="card stack-sm packet-summary" aria-label="Packet summary">
        <div>
          <strong>{formatTrack(detail.track)}</strong>
          <p className="subtle">Track</p>
        </div>
        <div>
          <strong>{detail.fitBreakdown.total}/100</strong>
          <p className="subtle">Fit</p>
        </div>
        <div>
          <strong>{detail.truthReport.passed ? "Passed" : "Review"}</strong>
          <p className="subtle">Truth Gate</p>
        </div>
      </section>

      <section className="card stack-sm packet-artifacts">
        <div className="row between wrap">
          <h4>{selected.fileName}</h4>
          <span className="chip">{selected.kind === "diff" ? "diff" : "markdown"}</span>
        </div>
        <div className="artifact-tabs" role="tablist" aria-label="Packet artifacts">
          {artifacts.map((artifact) => (
            <button
              key={artifact.id}
              type="button"
              role="tab"
              aria-selected={selected.id === artifact.id}
              className={`artifact-tab ${selected.id === artifact.id ? "active" : ""}`}
              onClick={() => setSelectedId(artifact.id)}
            >
              {artifact.label}
            </button>
          ))}
        </div>
        <div className="artifact-preview">
          {selected.kind === "diff" ? (
            <DiffViewer diff={selected.diff ?? ""} framed={false} />
          ) : (
            <MarkdownViewer markdown={selected.markdown ?? ""} />
          )}
        </div>
      </section>
    </div>
  );
}
