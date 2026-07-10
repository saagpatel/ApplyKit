//! Ed25519 signing for VAP packet manifests (Phase 1b).
//!
//! A per-install keypair is generated on first use and persisted at
//! `config/signing_key.hex` (0600). The private key never leaves disk; the
//! public key is embedded in every signed manifest so a consumer can verify
//! integrity, and its SHA-256 fingerprint (`public_key_id`) lets a consumer pin
//! provenance to a trusted key.
//!
//! The signature covers [`crate::manifest::signing_payload`] — a deterministic,
//! formatting-independent digest of the schema, identity, artifact hashes, and
//! truth verdict — so an independent verifier (JobCommandCenter) reconstructs the
//! exact signed bytes without agreeing on JSON serialization.

use std::path::Path;

use anyhow::{anyhow, Context};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

use crate::manifest::{sha256_hex, signing_payload, ManifestSignature, VapManifest};

/// Filename of the persisted private key seed, under the config directory.
pub const KEY_FILENAME: &str = "signing_key.hex";
const ALG: &str = "ed25519";

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn from_hex(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return Err("odd-length hex string".to_string());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

/// A loaded signing keypair.
pub struct PacketSigner {
    key: SigningKey,
}

impl PacketSigner {
    /// Construct directly from a 32-byte seed (used in tests for determinism).
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        PacketSigner { key: SigningKey::from_bytes(seed) }
    }

    /// Load the persisted keypair from `config_dir`, generating and saving a new
    /// one (0600) on first use.
    pub fn load_or_create(config_dir: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(config_dir)
            .with_context(|| format!("creating config dir {}", config_dir.display()))?;
        let path = config_dir.join(KEY_FILENAME);

        let seed: [u8; 32] = if path.exists() {
            let hex = std::fs::read_to_string(&path)
                .with_context(|| format!("reading signing key {}", path.display()))?;
            from_hex(&hex)
                .map_err(|e| anyhow!("signing key {} is not valid hex: {e}", path.display()))?
                .try_into()
                .map_err(|_| anyhow!("signing key {} must be 32 bytes", path.display()))?
        } else {
            let mut seed = [0u8; 32];
            getrandom::getrandom(&mut seed).map_err(|e| anyhow!("generating signing key: {e}"))?;
            std::fs::write(&path, to_hex(&seed))
                .with_context(|| format!("writing signing key {}", path.display()))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&path)?.permissions();
                perms.set_mode(0o600);
                std::fs::set_permissions(&path, perms)?;
            }
            seed
        };

        Ok(PacketSigner { key: SigningKey::from_bytes(&seed) })
    }

    fn verifying_key(&self) -> VerifyingKey {
        self.key.verifying_key()
    }

    /// Hex of the 32-byte Ed25519 public key.
    pub fn public_key_hex(&self) -> String {
        to_hex(self.verifying_key().to_bytes().as_slice())
    }

    /// SHA-256 fingerprint of the public key (a consumer pins this for provenance).
    pub fn public_key_id(&self) -> String {
        sha256_hex(self.verifying_key().to_bytes().as_slice())
    }

    /// Sign a manifest in place, populating its `signature` block.
    pub fn sign_manifest(&self, manifest: &mut VapManifest) {
        let signature: Signature = self.key.sign(&signing_payload(manifest));
        manifest.signature = Some(ManifestSignature {
            alg: ALG.to_string(),
            public_key: self.public_key_hex(),
            public_key_id: self.public_key_id(),
            signature: to_hex(&signature.to_bytes()),
        });
    }
}

/// Verify a manifest's embedded Ed25519 signature against its own public key.
/// Returns true only if the signature block is present, well-formed, and valid
/// over [`signing_payload`]. This proves integrity (the manifest and artifact
/// hashes are unaltered); pin `public_key_id` separately to assert provenance.
pub fn verify_manifest_signature(manifest: &VapManifest) -> bool {
    let Some(block) = &manifest.signature else {
        return false;
    };
    if block.alg != ALG {
        return false;
    }
    // Fingerprint must actually match the embedded key.
    if sha256_hex(&from_hex(&block.public_key).unwrap_or_default()) != block.public_key_id {
        return false;
    }
    let Ok(pk_bytes) = from_hex(&block.public_key) else {
        return false;
    };
    let Ok(pk_arr): Result<[u8; 32], _> = pk_bytes.try_into() else {
        return false;
    };
    let Ok(verifying_key) = VerifyingKey::from_bytes(&pk_arr) else {
        return false;
    };
    let Ok(sig_bytes) = from_hex(&block.signature) else {
        return false;
    };
    let Ok(sig_arr): Result<[u8; 64], _> = sig_bytes.try_into() else {
        return false;
    };
    let signature = Signature::from_bytes(&sig_arr);
    verifying_key.verify(&signing_payload(manifest), &signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{
        build_manifest, ManifestArtifact, ManifestFit, ManifestInputs, ManifestSource,
        ManifestTruth,
    };

    fn sample_manifest() -> VapManifest {
        build_manifest(ManifestInputs {
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
                path: "Resume_1pg_Tailored.md".to_string(),
                sha256: "aa".to_string(),
                format: "md".to_string(),
            }],
        })
    }

    #[test]
    fn sign_then_verify_roundtrip() {
        let signer = PacketSigner::from_seed(&[7u8; 32]);
        let mut manifest = sample_manifest();
        signer.sign_manifest(&mut manifest);

        let block = manifest.signature.clone().expect("signed");
        assert_eq!(block.alg, "ed25519");
        assert_eq!(block.public_key, signer.public_key_hex());
        assert_eq!(block.public_key_id, signer.public_key_id());
        assert!(verify_manifest_signature(&manifest));
    }

    #[test]
    fn ed25519_signing_is_deterministic() {
        let signer = PacketSigner::from_seed(&[7u8; 32]);
        let mut a = sample_manifest();
        let mut b = sample_manifest();
        signer.sign_manifest(&mut a);
        signer.sign_manifest(&mut b);
        assert_eq!(a.signature, b.signature, "same key + payload => same signature");
    }

    #[test]
    fn tampering_an_artifact_hash_breaks_the_signature() {
        let signer = PacketSigner::from_seed(&[7u8; 32]);
        let mut manifest = sample_manifest();
        signer.sign_manifest(&mut manifest);
        // Edit an artifact hash after signing (as a downstream tamper would).
        manifest.artifacts[0].sha256 = "ff".to_string();
        assert!(!verify_manifest_signature(&manifest), "tampered manifest must fail verify");
    }

    #[test]
    fn tampering_a_custom_field_breaks_the_signature() {
        let signer = PacketSigner::from_seed(&[7u8; 32]);
        let mut manifest = sample_manifest();
        manifest.custom_fields.insert("desired_salary".to_string(), "180000".to_string());
        signer.sign_manifest(&mut manifest);
        manifest.custom_fields.insert("desired_salary".to_string(), "190000".to_string());
        assert!(!verify_manifest_signature(&manifest));
    }

    #[test]
    fn flipping_truth_verdict_breaks_the_signature() {
        let signer = PacketSigner::from_seed(&[7u8; 32]);
        let mut manifest = sample_manifest();
        signer.sign_manifest(&mut manifest);
        manifest.truth.passed = false;
        assert!(!verify_manifest_signature(&manifest));
    }

    #[test]
    fn load_or_create_persists_same_key() {
        let dir = tempfile::tempdir().expect("tmp");
        let first = PacketSigner::load_or_create(dir.path()).expect("create");
        let pubkey = first.public_key_hex();
        assert!(dir.path().join(KEY_FILENAME).exists());
        let second = PacketSigner::load_or_create(dir.path()).expect("load");
        assert_eq!(pubkey, second.public_key_hex(), "persisted key must be stable");
    }
}
