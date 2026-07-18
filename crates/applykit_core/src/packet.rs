use crate::types::{ExtractedJd, FitScore, TailorPlan, Track, TrackerRow};
use anyhow::Context;
use chrono::NaiveDate;
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct StagedPacket {
    pub temp_dir: PathBuf,
    pub final_dir: PathBuf,
    pub staged_files: Vec<PathBuf>,
    pub final_files: Vec<PathBuf>,
    pub tracker_row: TrackerRow,
}

#[derive(Debug, Clone)]
pub struct PacketWriteInput<'a> {
    pub output_base: &'a Path,
    pub date: NaiveDate,
    pub company: &'a str,
    pub role: &'a str,
    pub identity_suffix: &'a str,
    pub source: &'a str,
    pub jd_text: &'a str,
    pub extracted: &'a ExtractedJd,
    pub fit: &'a FitScore,
    pub tailor_plan: &'a TailorPlan,
    pub resume_1pg: &'a str,
    pub resume_2pg: Option<&'a str>,
    pub recruiter_message: &'a str,
    pub hiring_manager_message: &'a str,
    pub cover_short_message: &'a str,
    pub diff_md: &'a str,
    pub track: Track,
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else if ch.is_whitespace() || ch == '-' || ch == '_' {
            out.push('_');
        }
    }
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    out.trim_matches('_').to_string()
}

fn write_file(path: &Path, content: &str) -> anyhow::Result<()> {
    let mut file =
        fs::File::create(path).with_context(|| format!("creating {}", path.display()))?;
    file.write_all(content.as_bytes()).with_context(|| format!("writing {}", path.display()))?;
    file.sync_all().with_context(|| format!("syncing {}", path.display()))?;
    Ok(())
}

fn sync_dir(path: &Path) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        let dir = fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
        dir.sync_all().with_context(|| format!("syncing {}", path.display()))?;
    }
    Ok(())
}

fn fitscore_md(fit: &FitScore) -> String {
    let mut out = String::new();
    out.push_str("# Fit Score\n\n");
    out.push_str(&format!("Total: **{} / 100**\n\n", fit.total));
    out.push_str("## Breakdown\n");
    out.push_str(&format!("- Role match: {}\n", fit.role_match));
    out.push_str(&format!("- Stack match: {}\n", fit.stack_match));
    out.push_str(&format!("- Scale match: {}\n", fit.scale_match));
    out.push_str(&format!("- Rigor match: {}\n", fit.rigor_match));
    out.push_str(&format!("- Signal boost: {}\n\n", fit.signal_boost));

    out.push_str("## Why You Match\n");
    for row in &fit.why_match {
        out.push_str("- ");
        out.push_str(row);
        out.push('\n');
    }
    out.push('\n');

    out.push_str("## Gaps\n");
    if fit.gaps.is_empty() {
        out.push_str("- No critical gaps detected from explicit JD requirements.\n");
    } else {
        for row in &fit.gaps {
            out.push_str("- ");
            out.push_str(row);
            out.push('\n');
        }
    }
    out
}

fn tailor_plan_md(plan: &TailorPlan) -> String {
    let mut out = String::new();
    out.push_str("# Tailor Plan\n\n");
    out.push_str(&format!("- Max edits: {}\n", plan.max_resume_edits));
    out.push_str(&format!("- Max bullet swaps: {}\n\n", plan.max_bullet_swaps));
    for (idx, edit) in plan.edits.iter().enumerate() {
        out.push_str(&format!("{}. {} [{}]\n", idx + 1, edit.reason, edit.target_section));
    }
    out
}

fn tracker_csv(row: &TrackerRow) -> String {
    fn csv_escape(value: &str) -> String {
        if value.contains(',')
            || value.contains('"')
            || value.contains('\n')
            || value.contains('\r')
        {
            format!("\"{}\"", value.replace('"', "\"\""))
        } else {
            value.to_string()
        }
    }

    let mut out = String::new();
    out.push_str("date,company,role,source,track,fit_total,status,next_action,packet_dir\n");
    out.push_str(&format!(
        "{},{},{},{},{},{},{},{},{}\n",
        csv_escape(&row.date),
        csv_escape(&row.company),
        csv_escape(&row.role),
        csv_escape(&row.source),
        csv_escape(&row.track),
        row.fit_total,
        csv_escape(&row.status),
        csv_escape(&row.next_action),
        csv_escape(&row.packet_dir)
    ));
    out
}

pub fn stage_packet(input: PacketWriteInput<'_>) -> anyhow::Result<StagedPacket> {
    fs::create_dir_all(input.output_base)
        .with_context(|| format!("creating output base {}", input.output_base.display()))?;

    let folder_name = format!(
        "{}_{}_{}_{}",
        slugify(input.company),
        slugify(input.role),
        input.date.format("%Y-%m-%d"),
        input.identity_suffix
    );
    let final_dir = input.output_base.join(folder_name);
    let sequence = STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let tmp_dir = input.output_base.join(format!(
        ".{}.tmp-{}-{sequence}",
        final_dir.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id()
    ));
    fs::create_dir_all(&tmp_dir).with_context(|| format!("creating {}", tmp_dir.display()))?;

    let mut files_written = Vec::new();

    let mut write_named = |name: &str, content: &str| -> anyhow::Result<()> {
        let path = tmp_dir.join(name);
        write_file(&path, content)?;
        files_written.push(path);
        Ok(())
    };

    write_named("JD.txt", input.jd_text)?;
    let extracted_json = serde_json::to_string_pretty(input.extracted)?;
    write_named("Extracted.json", &extracted_json)?;
    write_named("FitScore.md", &fitscore_md(input.fit))?;
    write_named("TailorPlan.md", &tailor_plan_md(input.tailor_plan))?;
    write_named("Resume_1pg_Tailored.md", input.resume_1pg)?;
    if let Some(two_page) = input.resume_2pg {
        write_named("Resume_2pg_Tailored.md", two_page)?;
    }
    write_named("RecruiterMessage.md", input.recruiter_message)?;
    write_named("HiringManagerMessage.md", input.hiring_manager_message)?;
    write_named("CoverNote_Short.md", input.cover_short_message)?;
    write_named("Diff.md", input.diff_md)?;
    let tracker_row = TrackerRow {
        date: input.date.format("%Y-%m-%d").to_string(),
        company: input.company.to_string(),
        role: input.role.to_string(),
        source: input.source.to_string(),
        track: input.track.to_string(),
        fit_total: input.fit.total,
        status: "new".to_string(),
        next_action: String::new(),
        packet_dir: final_dir.to_string_lossy().to_string(),
    };
    write_named("TrackerRow.csv", &tracker_csv(&tracker_row))?;

    let meta_json = serde_json::to_string_pretty(&json!({
        "company": input.company,
        "role": input.role,
        "date": input.date.format("%Y-%m-%d").to_string(),
        "track": input.track,
        "fit_total": input.fit.total
    }))?;
    write_named("Meta.json", &meta_json)?;
    sync_dir(&tmp_dir)?;

    let final_files: Vec<PathBuf> = files_written
        .into_iter()
        .map(|tmp_path| {
            let file_name =
                tmp_path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            final_dir.join(file_name)
        })
        .collect();

    let staged_files = final_files
        .iter()
        .filter_map(|path| path.file_name().map(|name| tmp_dir.join(name)))
        .collect();
    Ok(StagedPacket { temp_dir: tmp_dir, final_dir, staged_files, final_files, tracker_row })
}

pub fn publish_staged_packet(staged: &StagedPacket) -> anyhow::Result<Option<PathBuf>> {
    let backup_dir = staged.final_dir.with_file_name(format!(
        ".{}.backup-{}",
        staged.final_dir.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id()
    ));
    if backup_dir.exists() {
        anyhow::bail!("packet backup already exists: {}", backup_dir.display());
    }
    let backup = if staged.final_dir.exists() {
        fs::rename(&staged.final_dir, &backup_dir).with_context(|| {
            format!(
                "preserving existing packet {} as {}",
                staged.final_dir.display(),
                backup_dir.display()
            )
        })?;
        Some(backup_dir)
    } else {
        None
    };
    if let Err(error) = fs::rename(&staged.temp_dir, &staged.final_dir) {
        if let Some(backup) = &backup {
            let _ = fs::rename(backup, &staged.final_dir);
        }
        return Err(error).with_context(|| {
            format!(
                "publishing staged packet {} -> {}",
                staged.temp_dir.display(),
                staged.final_dir.display()
            )
        });
    }
    if let Err(sync_error) = sync_dir(staged.final_dir.parent().unwrap_or(Path::new(".")))
        .and_then(|()| sync_dir(&staged.final_dir))
    {
        let mut recovery_errors = Vec::new();
        if let Err(error) = fs::rename(&staged.final_dir, &staged.temp_dir) {
            recovery_errors.push(format!("restaging failed: {error}"));
        }
        if let Some(backup) = &backup {
            if let Err(error) = fs::rename(backup, &staged.final_dir) {
                recovery_errors.push(format!("backup restore failed: {error}"));
            }
        }
        let recovery = if recovery_errors.is_empty() {
            "previous packet state restored".to_string()
        } else {
            recovery_errors.join("; ")
        };
        anyhow::bail!("syncing published packet failed: {sync_error}; {recovery}");
    }
    Ok(backup)
}

pub fn rollback_published_packet(
    staged: &StagedPacket,
    backup: Option<&Path>,
) -> anyhow::Result<PathBuf> {
    let quarantine = staged.final_dir.with_file_name(format!(
        ".{}.recovery-{}",
        staged.final_dir.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id()
    ));
    fs::rename(&staged.final_dir, &quarantine).with_context(|| {
        format!(
            "quarantining incomplete packet {} as {}",
            staged.final_dir.display(),
            quarantine.display()
        )
    })?;
    if let Some(backup) = backup {
        fs::rename(backup, &staged.final_dir).with_context(|| {
            format!(
                "restoring previous packet {} -> {}",
                backup.display(),
                staged.final_dir.display()
            )
        })?;
    }
    sync_dir(staged.final_dir.parent().unwrap_or(Path::new(".")))?;
    Ok(quarantine)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracker_csv_escapes_commas_and_quotes() {
        let row = TrackerRow {
            date: "2026-02-14".to_string(),
            company: "Acme, Inc.".to_string(),
            role: "Senior \"Support\" Engineer".to_string(),
            source: "manual".to_string(),
            track: "Support/Ops Core".to_string(),
            fit_total: 55,
            status: "new".to_string(),
            next_action: "Follow up,\nMonday".to_string(),
            packet_dir: "/tmp/packets/Acme, Inc".to_string(),
        };

        let csv = tracker_csv(&row);
        assert!(csv.contains("\"Acme, Inc.\""));
        assert!(csv.contains("\"Senior \"\"Support\"\" Engineer\""));
        assert!(csv.contains("\"Follow up,\nMonday\""));
    }
}
