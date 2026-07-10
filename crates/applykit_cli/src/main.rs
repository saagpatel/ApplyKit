use anyhow::Context;
use applykit_core::types::{Baseline, GenerateInput, Track};
use applykit_core::{generate_packet, GenerateOptions};
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "applykit")]
#[command(version)]
#[command(about = "ApplyKit local-first packet generator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Generate {
        #[arg(long)]
        company: String,
        #[arg(long)]
        role: String,
        #[arg(long)]
        source: String,
        #[arg(long)]
        baseline: String,
        #[arg(long)]
        jd: PathBuf,
        #[arg(long)]
        outdir: Option<PathBuf>,
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        track_override: Option<String>,
        #[arg(long, default_value_t = false)]
        allow_unapproved: bool,
        /// JSON object of operator-approved ATS answers keyed by field name or label.
        #[arg(long)]
        custom_fields: Option<PathBuf>,
    },
}

fn load_custom_fields(path: Option<&Path>) -> anyhow::Result<BTreeMap<String, String>> {
    match path {
        Some(path) => serde_json::from_str::<BTreeMap<String, String>>(
            &std::fs::read_to_string(path)
                .with_context(|| format!("reading custom fields file {}", path.display()))?,
        )
        .with_context(|| format!("parsing custom fields file {}", path.display())),
        None => Ok(BTreeMap::new()),
    }
}

fn attach_custom_fields(
    packet_dir: &Path,
    config_dir: &Path,
    custom_fields: BTreeMap<String, String>,
) -> anyhow::Result<()> {
    if custom_fields.is_empty() {
        return Ok(());
    }

    let manifest_path = packet_dir.join(applykit_core::manifest::MANIFEST_FILENAME);
    let raw = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("reading packet manifest {}", manifest_path.display()))?;
    let mut manifest: applykit_core::manifest::VapManifest = serde_json::from_str(&raw)
        .with_context(|| format!("parsing packet manifest {}", manifest_path.display()))?;
    manifest.custom_fields = custom_fields;
    let signer = applykit_core::signing::PacketSigner::load_or_create(config_dir)
        .context("loading ApplyKit packet signer")?;
    signer.sign_manifest(&mut manifest);
    applykit_core::manifest::write_manifest_file(packet_dir, &manifest)
        .context("writing signed packet custom fields")
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().context("reading cwd")?;

    match cli.command {
        Commands::Generate {
            company,
            role,
            source,
            baseline,
            jd,
            outdir,
            date,
            track_override,
            allow_unapproved,
            custom_fields,
        } => {
            let jd_text = std::fs::read_to_string(&jd)
                .with_context(|| format!("reading JD file {}", jd.display()))?;
            let baseline = baseline.parse::<Baseline>().map_err(anyhow::Error::msg)?;
            let date = match date {
                Some(v) => Some(NaiveDate::parse_from_str(&v, "%Y-%m-%d")?),
                None => None,
            };
            let track_override = match track_override {
                Some(v) => Some(v.parse::<Track>().map_err(anyhow::Error::msg)?),
                None => None,
            };
            let custom_fields = load_custom_fields(custom_fields.as_deref())?;

            let result = generate_packet(
                GenerateInput {
                    company,
                    role,
                    source,
                    baseline,
                    jd_text,
                    outdir,
                    run_date: date,
                    track_override,
                    allow_unapproved,
                },
                GenerateOptions { repo_root: cwd.clone() },
            )?;

            attach_custom_fields(&result.packet_dir, &cwd.join("config"), custom_fields)?;

            println!("Packet generated successfully");
            println!("Track: {}", result.track.selected);
            println!("Fit Score: {}", result.fit.total);
            println!("Output Dir: {}", result.packet_dir.display());
            println!("Files:");
            for path in result.files_written {
                println!("- {}", path.display());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use applykit_core::manifest::{
        build_manifest, ManifestArtifact, ManifestFit, ManifestInputs, ManifestSource,
        ManifestTruth,
    };

    fn sample_manifest() -> applykit_core::manifest::VapManifest {
        build_manifest(ManifestInputs {
            generated_at: "2026-07-10T00:00:00Z".to_string(),
            generator_version: "0.1.0".to_string(),
            git_sha: None,
            source: ManifestSource {
                company: "Acme".to_string(),
                role: "Engineer".to_string(),
                jd_sha256: "deadbeef".to_string(),
                source_platform: "manual".to_string(),
            },
            truth: ManifestTruth {
                method: "deterministic-rule-gate/v1".to_string(),
                gate_version: "1".to_string(),
                passed: true,
                provenance_complete: true,
                violations: vec![],
                unknown_tools: vec![],
                claim_issues: vec![],
            },
            fit: ManifestFit { track: "T".to_string(), total: 70, gaps: vec![] },
            artifacts: vec![ManifestArtifact {
                role: "resume_1pg".to_string(),
                path: "Resume.md".to_string(),
                sha256: "aa".to_string(),
                format: "md".to_string(),
            }],
        })
    }

    #[test]
    fn load_custom_fields_requires_a_string_map() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fields_path = dir.path().join("fields.json");
        std::fs::write(&fields_path, r#"{"desired_salary":"180000"}"#).expect("write fields");

        assert_eq!(
            load_custom_fields(Some(&fields_path)).expect("parse fields"),
            BTreeMap::from([("desired_salary".to_string(), "180000".to_string())])
        );
        assert!(load_custom_fields(None).expect("empty fields").is_empty());
    }

    #[test]
    fn attach_custom_fields_signs_the_updated_manifest() {
        let dir = tempfile::tempdir().expect("tempdir");
        let packet_dir = dir.path().join("packet");
        std::fs::create_dir(&packet_dir).expect("packet dir");
        let manifest = sample_manifest();
        applykit_core::manifest::write_manifest_file(&packet_dir, &manifest)
            .expect("write manifest");

        attach_custom_fields(
            &packet_dir,
            &dir.path().join("config"),
            BTreeMap::from([("desired_salary".to_string(), "180000".to_string())]),
        )
        .expect("attach fields");

        let raw =
            std::fs::read_to_string(packet_dir.join(applykit_core::manifest::MANIFEST_FILENAME))
                .expect("read manifest");
        let updated: applykit_core::manifest::VapManifest =
            serde_json::from_str(&raw).expect("parse manifest");
        assert_eq!(updated.custom_fields.get("desired_salary"), Some(&"180000".to_string()));
        assert!(applykit_core::signing::verify_manifest_signature(&updated));
    }
}
