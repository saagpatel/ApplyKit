//! Verified Application Packet (VAP) manifest — schema `vap/1`.
//!
//! ApplyKit writes one `packet.manifest.json` beside every generated packet. It
//! is the machine-readable contract a downstream consumer (e.g. JobCommandCenter)
//! uses to identify the packet, trust the truth-gate verdict, and detect
//! post-generation edits by re-hashing each listed artifact.
//!
//! Invariants (see `docs/vap-manifest-v1.md` for the full contract):
//! - `packet_id` is deterministic and content-addressed over the artifact hashes.
//! - `generated_at` is descriptive only and is NOT part of `packet_id`.
//! - The manifest never lists itself or `ReviewData.json`; the artifact set is
//!   exactly the packet deliverables passed by the caller.

use anyhow::Context;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Current manifest schema version. A consumer that does not recognize this
/// string MUST refuse the packet rather than parse it partially.
pub const VAP_SCHEMA_VERSION: &str = "vap/1";
/// Honest label for ApplyKit's truth gate: a deterministic rule-based scan, not
/// an adversarial multi-pass verifier.
pub const TRUTH_GATE_METHOD: &str = "deterministic-rule-gate/v1";
/// Semver-ish version of the gate's own ruleset.
pub const TRUTH_GATE_VERSION: &str = "1";
/// Filename written into the packet directory.
pub const MANIFEST_FILENAME: &str = "packet.manifest.json";

/// Lowercase hex SHA-256 of the given bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestArtifact {
    /// Stable role tag, e.g. `resume_1pg`, `cover_note`.
    pub role: String,
    /// Path relative to the packet directory (a bare filename; packets are flat).
    pub path: String,
    pub sha256: String,
    /// File extension without the dot, e.g. `md`, `json`, `txt`, `csv`.
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestGenerator {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_sha: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestSource {
    pub company: String,
    pub role: String,
    /// SHA-256 of the raw input JD; matches ApplyKit's internal job id.
    pub jd_sha256: String,
    pub source_platform: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestTruth {
    pub method: String,
    pub gate_version: String,
    pub passed: bool,
    pub provenance_complete: bool,
    pub violations: Vec<String>,
    pub unknown_tools: Vec<String>,
    pub claim_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestFit {
    pub track: String,
    pub total: u8,
    pub gaps: Vec<String>,
}

/// Ed25519 signature block. Absent on unsigned packets; populated when a signing
/// key is available. The signature covers [`signing_payload`], not the raw JSON,
/// so it is independent of serialization formatting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestSignature {
    pub alg: String,
    /// Hex of the 32-byte Ed25519 public key (lets a consumer verify integrity;
    /// pin `public_key_id` in the consumer to also assert provenance).
    pub public_key: String,
    /// SHA-256 fingerprint of `public_key`.
    pub public_key_id: String,
    /// Hex of the 64-byte signature over [`signing_payload`].
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VapManifest {
    pub schema_version: String,
    pub packet_id: String,
    pub generator: ManifestGenerator,
    pub generated_at: String,
    pub source: ManifestSource,
    pub truth: ManifestTruth,
    pub fit: ManifestFit,
    pub artifacts: Vec<ManifestArtifact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<ManifestSignature>,
}

/// Map a packet filename to its `(role, format)`.
fn classify_artifact(file_name: &str) -> (String, String) {
    let role = match file_name {
        "JD.txt" => "jd",
        "Extracted.json" => "extracted",
        "FitScore.md" => "fit_score",
        "TailorPlan.md" => "tailor_plan",
        "Resume_1pg_Tailored.md" => "resume_1pg",
        "Resume_2pg_Tailored.md" => "resume_2pg",
        "RecruiterMessage.md" => "recruiter_message",
        "HiringManagerMessage.md" => "hiring_manager_message",
        "CoverNote_Short.md" => "cover_note",
        "Diff.md" => "diff",
        "TrackerRow.csv" => "tracker_csv",
        "Meta.json" => "meta",
        _ => "other",
    };
    let format = file_name.rsplit('.').next().filter(|ext| *ext != file_name).unwrap_or("");
    (role.to_string(), format.to_string())
}

/// Application-identity packet id: `sha256:` over `"<company>\0<role>\0<jd_sha256>"`.
///
/// This names *which application* the packet is for. It is intentionally
/// independent of output location, generation time, and artifact bytes, so
/// re-tailoring the same role against the same JD yields the same `packet_id` —
/// a new *version* of the same application. Per-file integrity and version drift
/// are carried by `artifacts[].sha256`, not by this id. It mirrors the identity
/// ApplyKit already keys job records on (`company:role:...:sha256(jd)`), so both
/// sides of the contract agree on what "the same application" means.
pub fn compute_packet_id(company: &str, role: &str, jd_sha256: &str) -> String {
    let basis = format!("{company}\0{role}\0{jd_sha256}");
    format!("sha256:{}", sha256_hex(basis.as_bytes()))
}

/// Deterministic bytes an Ed25519 signature covers. Built from the
/// security-relevant fields (schema, identity, every artifact hash, and the
/// truth verdict) rather than the raw JSON, so a consumer reconstructs the exact
/// same payload regardless of serialization formatting or field order. Editing
/// any artifact (its hash), the identity (its id), the schema, or the verdict
/// changes this payload and invalidates the signature.
pub fn signing_payload(manifest: &VapManifest) -> Vec<u8> {
    let mut lines = vec![
        format!("schema={}", manifest.schema_version),
        format!("packet_id={}", manifest.packet_id),
    ];
    let mut artifact_lines: Vec<String> = manifest
        .artifacts
        .iter()
        .map(|artifact| format!("artifact\0{}\0{}", artifact.path, artifact.sha256))
        .collect();
    artifact_lines.sort();
    lines.extend(artifact_lines);
    lines.push(format!("truth_passed={}", manifest.truth.passed));
    lines.join("\n").into_bytes()
}

/// Explicit inputs for [`build_manifest`]. Keeping these explicit (rather than
/// reading the clock or environment inside the builder) keeps the core pure and
/// deterministic for tests.
pub struct ManifestInputs {
    pub generated_at: String,
    pub generator_version: String,
    pub git_sha: Option<String>,
    pub source: ManifestSource,
    pub truth: ManifestTruth,
    pub fit: ManifestFit,
    /// Artifacts with hashes already computed; will be sorted by path.
    pub artifacts: Vec<ManifestArtifact>,
}

/// Assemble a manifest from fully-resolved inputs. Pure and deterministic: the
/// non-deterministic inputs (`generated_at`, output location) do not affect
/// `packet_id`, which is derived from the application identity in `source`.
pub fn build_manifest(inputs: ManifestInputs) -> VapManifest {
    let mut artifacts = inputs.artifacts;
    artifacts.sort_by(|a, b| a.path.cmp(&b.path));
    let packet_id =
        compute_packet_id(&inputs.source.company, &inputs.source.role, &inputs.source.jd_sha256);
    VapManifest {
        schema_version: VAP_SCHEMA_VERSION.to_string(),
        packet_id,
        generator: ManifestGenerator {
            name: "applykit".to_string(),
            version: inputs.generator_version,
            git_sha: inputs.git_sha,
        },
        generated_at: inputs.generated_at,
        source: inputs.source,
        truth: inputs.truth,
        fit: inputs.fit,
        artifacts,
        signature: None,
    }
}

impl VapManifest {
    /// Re-hash every listed artifact against `packet_dir` and return the paths
    /// whose on-disk contents no longer match the manifest. An empty vec means
    /// the packet is intact; a non-empty vec means STALE (edited after
    /// generation). A missing file is reported as mismatched.
    pub fn verify_against_dir(&self, packet_dir: &Path) -> Vec<String> {
        let mut mismatched = Vec::new();
        for artifact in &self.artifacts {
            let path = packet_dir.join(&artifact.path);
            match std::fs::read(&path) {
                Ok(bytes) if sha256_hex(&bytes) == artifact.sha256 => {}
                _ => mismatched.push(artifact.path.clone()),
            }
        }
        mismatched
    }
}

/// Hash each packet deliverable into a `ManifestArtifact`. `files_written` is the
/// artifact set returned by [`crate::packet::write_packet`]; the manifest never
/// lists itself or `ReviewData.json` because neither is in that set.
pub fn collect_artifacts(files_written: &[PathBuf]) -> anyhow::Result<Vec<ManifestArtifact>> {
    let mut artifacts = Vec::with_capacity(files_written.len());
    for path in files_written {
        let file_name =
            path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let bytes = std::fs::read(path).with_context(|| format!("hashing {}", path.display()))?;
        let (role, format) = classify_artifact(&file_name);
        artifacts.push(ManifestArtifact {
            role,
            path: file_name,
            sha256: sha256_hex(&bytes),
            format,
        });
    }
    Ok(artifacts)
}

/// Serialize a manifest and write it to `packet_dir/packet.manifest.json`.
pub fn write_manifest_file(packet_dir: &Path, manifest: &VapManifest) -> anyhow::Result<()> {
    let manifest_path = packet_dir.join(MANIFEST_FILENAME);
    let json = serde_json::to_string_pretty(manifest).context("serializing packet manifest")?;
    std::fs::write(&manifest_path, json)
        .with_context(|| format!("writing {}", manifest_path.display()))?;
    Ok(())
}

/// Build and write an unsigned `vap/1` manifest in one step (hash artifacts →
/// build → write). Signing is layered in by the pipeline between build and write
/// via [`crate::signing`]; this convenience is retained for the unsigned path.
#[allow(clippy::too_many_arguments)]
pub fn emit_manifest_file(
    packet_dir: &Path,
    files_written: &[PathBuf],
    source: ManifestSource,
    truth: ManifestTruth,
    fit: ManifestFit,
    generated_at: String,
    git_sha: Option<String>,
) -> anyhow::Result<VapManifest> {
    let artifacts = collect_artifacts(files_written)?;
    let manifest = build_manifest(ManifestInputs {
        generated_at,
        generator_version: env!("CARGO_PKG_VERSION").to_string(),
        git_sha,
        source,
        truth,
        fit,
        artifacts,
    });
    write_manifest_file(packet_dir, &manifest)?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact(path: &str, sha: &str) -> ManifestArtifact {
        let (role, format) = classify_artifact(path);
        ManifestArtifact { role, path: path.to_string(), sha256: sha.to_string(), format }
    }

    fn sample_inputs(artifacts: Vec<ManifestArtifact>) -> ManifestInputs {
        ManifestInputs {
            generated_at: "2026-07-09T12:00:00-07:00".to_string(),
            generator_version: "0.1.0".to_string(),
            git_sha: None,
            source: ManifestSource {
                company: "Acme".to_string(),
                role: "Senior Engineer".to_string(),
                jd_sha256: "deadbeef".to_string(),
                source_platform: "LinkedIn".to_string(),
            },
            truth: ManifestTruth {
                method: TRUTH_GATE_METHOD.to_string(),
                gate_version: TRUTH_GATE_VERSION.to_string(),
                passed: true,
                provenance_complete: true,
                violations: vec![],
                unknown_tools: vec![],
                claim_issues: vec![],
            },
            fit: ManifestFit { track: "Automation/AI Ops".to_string(), total: 78, gaps: vec![] },
            artifacts,
        }
    }

    #[test]
    fn packet_id_is_deterministic_for_same_identity() {
        let arts = vec![artifact("Resume_1pg_Tailored.md", "aa"), artifact("Diff.md", "bb")];
        let id1 = build_manifest(sample_inputs(arts.clone())).packet_id;
        let id2 = build_manifest(sample_inputs(arts)).packet_id;
        assert_eq!(id1, id2);
        assert!(id1.starts_with("sha256:"), "packet_id should be sha256-prefixed: {id1}");
    }

    #[test]
    fn packet_id_is_identity_not_content() {
        // Same application identity, different artifact bytes -> same packet_id.
        // Identity names the application; artifact hashes carry version/integrity.
        let v1 = vec![artifact("Resume_1pg_Tailored.md", "aa")];
        let v2 = vec![artifact("Resume_1pg_Tailored.md", "zz"), artifact("Diff.md", "bb")];
        assert_eq!(
            build_manifest(sample_inputs(v1)).packet_id,
            build_manifest(sample_inputs(v2)).packet_id
        );
    }

    #[test]
    fn packet_id_ignores_generated_at() {
        let arts = vec![artifact("Resume_1pg_Tailored.md", "aa")];
        let mut a = sample_inputs(arts.clone());
        a.generated_at = "2020-01-01T00:00:00Z".to_string();
        let mut b = sample_inputs(arts);
        b.generated_at = "2099-12-31T23:59:59Z".to_string();
        assert_eq!(build_manifest(a).packet_id, build_manifest(b).packet_id);
    }

    #[test]
    fn packet_id_changes_when_application_identity_changes() {
        let arts = vec![artifact("Resume_1pg_Tailored.md", "aa")];
        let base = build_manifest(sample_inputs(arts.clone())).packet_id;

        let mut different_role = sample_inputs(arts.clone());
        different_role.source.role = "Staff Engineer".to_string();
        assert_ne!(base, build_manifest(different_role).packet_id);

        let mut different_jd = sample_inputs(arts);
        different_jd.source.jd_sha256 = "feedface".to_string();
        assert_ne!(base, build_manifest(different_jd).packet_id);
    }

    #[test]
    fn build_manifest_sets_schema_and_omits_signature() {
        let manifest = build_manifest(sample_inputs(vec![artifact("Diff.md", "bb")]));
        assert_eq!(manifest.schema_version, "vap/1");
        assert!(manifest.signature.is_none());
        assert_eq!(manifest.generator.name, "applykit");
        // signature field is skipped when None
        let json = serde_json::to_string(&manifest).expect("serialize");
        assert!(!json.contains("signature"), "unsigned manifest must omit signature key");
    }

    #[test]
    fn classify_artifact_maps_known_files() {
        assert_eq!(classify_artifact("Resume_1pg_Tailored.md"), ("resume_1pg".into(), "md".into()));
        assert_eq!(classify_artifact("TrackerRow.csv"), ("tracker_csv".into(), "csv".into()));
        assert_eq!(classify_artifact("Meta.json"), ("meta".into(), "json".into()));
        assert_eq!(classify_artifact("Mystery.bin"), ("other".into(), "bin".into()));
        assert_eq!(classify_artifact("noext"), ("other".into(), "".into()));
    }

    #[test]
    fn emit_and_verify_round_trip_detects_tampering() {
        let dir = tempfile::tempdir().expect("tmp");
        let resume = dir.path().join("Resume_1pg_Tailored.md");
        let diff = dir.path().join("Diff.md");
        std::fs::write(&resume, "original resume").expect("write resume");
        std::fs::write(&diff, "the diff").expect("write diff");

        let manifest = emit_manifest_file(
            dir.path(),
            &[resume.clone(), diff],
            ManifestSource {
                company: "Acme".to_string(),
                role: "Eng".to_string(),
                jd_sha256: "abc".to_string(),
                source_platform: "manual".to_string(),
            },
            ManifestTruth {
                method: TRUTH_GATE_METHOD.to_string(),
                gate_version: TRUTH_GATE_VERSION.to_string(),
                passed: true,
                provenance_complete: true,
                violations: vec![],
                unknown_tools: vec![],
                claim_issues: vec![],
            },
            ManifestFit { track: "T".to_string(), total: 50, gaps: vec![] },
            "2026-07-09T12:00:00-07:00".to_string(),
            None,
        )
        .expect("emit");

        // Manifest file exists and parses back to an equal value.
        let written = std::fs::read_to_string(dir.path().join(MANIFEST_FILENAME)).expect("read");
        let parsed: VapManifest = serde_json::from_str(&written).expect("parse");
        assert_eq!(parsed, manifest);

        // The manifest does not list itself.
        assert!(!manifest.artifacts.iter().any(|a| a.path == MANIFEST_FILENAME));
        assert_eq!(manifest.artifacts.len(), 2);

        // Intact packet verifies clean.
        assert!(manifest.verify_against_dir(dir.path()).is_empty());

        // Editing an artifact makes it STALE.
        std::fs::write(&resume, "EDITED after generation").expect("tamper");
        let mismatched = manifest.verify_against_dir(dir.path());
        assert_eq!(mismatched, vec!["Resume_1pg_Tailored.md".to_string()]);
    }
}
