use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use xiuxian_wendao_core::WendaoResourceUri;

use crate::skill_runtime::SkillRuntimeResolver;
use crate::skill_runtime::manifest::{
    SkillManifest, SkillManifestScan, SkillNativeAliasMountReport, SkillNativeAliasSpec,
    SkillWorkflowType, compile_skill_manifest_aliases,
};

impl From<AuthorizedSkillManifestScan> for SkillManifestScan {
    fn from(auth: AuthorizedSkillManifestScan) -> Self {
        Self {
            discovered_paths: auth.discovered_paths,
            manifests: auth.manifests,
            issues: auth.issues,
        }
    }
}

use super::catalog::SkillIntentCatalog;
use super::report::{SkillAuthorityReport, build_authority_report};

/// Validation result for only the manifests explicitly authorized by root `SKILL.md` files.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AuthorizedSkillManifestScan {
    /// Concrete paths for every discovered authorized manifest candidate.
    pub discovered_paths: Vec<PathBuf>,
    /// Successfully parsed and validated authorized manifests.
    pub manifests: Vec<SkillManifest>,
    /// Human-readable issues for authorized manifests that could not be resolved or validated.
    pub issues: Vec<String>,
    /// Authority classification report used to derive the authorized manifest set.
    pub authority: SkillAuthorityReport,
}

/// Prepared native-alias payload derived from the authorized-manifest scan.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AuthorizedSkillNativeAliasScan {
    /// Pre-mount report with discovery, authority, and compile-stage diagnostics populated.
    pub report: SkillNativeAliasMountReport<SkillWorkflowType>,
    /// Runtime-ready alias specs that Daochang can mount into the native tool registry.
    pub compiled_specs: Vec<SkillNativeAliasSpec<SkillWorkflowType>>,
}

impl SkillRuntimeResolver {
    /// Compare `SKILL.md` intention links against physically mounted runtime manifests.
    ///
    /// # Errors
    ///
    /// Returns an error when the runtime-skill `LinkGraphIndex` cannot be built or when a
    /// mounted `SKILL.md` document cannot be reparsed for raw intent targets.
    pub fn audit_manifest_authority(&self) -> Result<SkillAuthorityReport> {
        let catalog = self.collect_manifest_intents()?;
        Ok(self.audit_manifest_authority_with_catalog(&catalog))
    }

    /// Compare physical manifests against a reusable skill intent catalog.
    #[must_use]
    pub fn audit_manifest_authority_with_catalog(
        &self,
        catalog: &SkillIntentCatalog,
    ) -> SkillAuthorityReport {
        let physical_manifests = self
            .list_manifest_uris()
            .into_iter()
            .collect::<BTreeSet<_>>();
        build_authority_report(&physical_manifests, catalog)
    }

    /// Discover and validate only the manifests explicitly authorized by root `SKILL.md` files.
    ///
    /// Authority mismatches remain in the returned report, while load and validation failures for
    /// authorized manifests are collected in `issues` just like `scan_manifests()`. Call
    /// [`Self::scan_authorized_manifests_with_catalog`] when you already hold a reusable
    /// [`SkillIntentCatalog`] built from cached link-graph indexes.
    ///
    /// # Errors
    ///
    /// Returns an error when authority auditing cannot build or traverse the runtime skill link
    /// graph. Validation errors for individual authorized manifests are reported in `issues`.
    pub fn scan_authorized_manifests(&self) -> Result<AuthorizedSkillManifestScan> {
        let catalog = self.collect_manifest_intents()?;
        Ok(self.scan_authorized_manifests_with_catalog(&catalog))
    }

    /// Discover and validate authorized manifests from a reusable skill intent catalog.
    #[must_use]
    pub fn scan_authorized_manifests_with_catalog(
        &self,
        catalog: &SkillIntentCatalog,
    ) -> AuthorizedSkillManifestScan {
        let authority = self.audit_manifest_authority_with_catalog(catalog);
        build_authorized_manifest_scan(self, authority)
    }

    /// Discover, validate, and precompile authorized native aliases for runtime mounting.
    ///
    /// # Errors
    ///
    /// Returns an error when authority auditing cannot build or traverse the runtime skill link
    /// graph. Validation and compile errors for individual manifests are retained in the returned
    /// report `issues` list.
    pub fn scan_authorized_native_aliases(
        &self,
        root: &Path,
    ) -> Result<AuthorizedSkillNativeAliasScan> {
        let catalog = self.collect_manifest_intents()?;
        Ok(self.scan_authorized_native_aliases_with_catalog(root, &catalog))
    }

    /// Discover and precompile authorized native aliases from a reusable intent catalog.
    #[must_use]
    pub fn scan_authorized_native_aliases_with_catalog(
        &self,
        root: &Path,
        catalog: &SkillIntentCatalog,
    ) -> AuthorizedSkillNativeAliasScan {
        let scan = self.scan_authorized_manifests_with_catalog(catalog);
        build_authorized_native_alias_scan(root, scan)
    }
}

fn build_authorized_manifest_scan(
    resolver: &SkillRuntimeResolver,
    authority: SkillAuthorityReport,
) -> AuthorizedSkillManifestScan {
    let authorized_manifests = authority.authorized_manifests.clone();
    let mut scan = AuthorizedSkillManifestScan {
        discovered_paths: Vec::with_capacity(authorized_manifests.len()),
        manifests: Vec::with_capacity(authorized_manifests.len()),
        issues: Vec::new(),
        authority,
    };

    for manifest_uri in authorized_manifests {
        let parsed_uri = match WendaoResourceUri::parse(manifest_uri.as_str()) {
            Ok(uri) => uri,
            Err(error) => {
                scan.issues.push(format!("{manifest_uri} -> {error}"));
                continue;
            }
        };
        let source_path = match resolver.resolve_parsed_uri(&parsed_uri) {
            Ok(path) => {
                scan.discovered_paths.push(path.clone());
                path
            }
            Err(error) => {
                scan.issues.push(format!("{manifest_uri} -> {error}"));
                continue;
            }
        };

        match resolver.load_skill_manifest(manifest_uri.as_str()) {
            Ok(manifest) => scan.manifests.push(manifest),
            Err(error) => scan
                .issues
                .push(format!("{} -> {error}", source_path.display())),
        }
    }

    scan
}

fn build_authorized_native_alias_scan(
    root: &Path,
    scan: AuthorizedSkillManifestScan,
) -> AuthorizedSkillNativeAliasScan {
    let AuthorizedSkillManifestScan {
        discovered_paths,
        manifests,
        issues,
        authority,
    } = scan;
    let compilation = compile_skill_manifest_aliases(manifests);

    let mut report = SkillNativeAliasMountReport::from_root(root);
    report.discovered_paths = discovered_paths;
    report.authorized_count = authority.authorized_manifests.len();
    report.ghost_count = authority.ghost_links.len();
    report.unauthorized_count = authority.unauthorized_manifests.len();
    report.issues.extend(issues);
    report.issues.extend(
        authority
            .ghost_links
            .iter()
            .map(|uri| format!("{uri} -> declared by SKILL.md but manifest is missing")),
    );
    report.issues.extend(
        authority
            .unauthorized_manifests
            .iter()
            .map(|uri| format!("{uri} -> manifest is present but not granted by root SKILL.md")),
    );
    report.issues.extend(compilation.issues);

    AuthorizedSkillNativeAliasScan {
        report,
        compiled_specs: compilation.compiled_specs,
    }
}
