use std::path::Path;

use super::support::collect_domain_contract_imports;

const DOMAIN_CONTRACT_IMPORT_HEAD: &str = "xiuxian_wendao::search";
const DOMAIN_CONTRACT_IMPORT_TAIL: &str = "::contracts";

#[test]
fn studio_code_uses_studio_contract_import_path() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let allowed_reexports = [
        manifest_dir.join("src/contracts/types.rs"),
        manifest_dir.join("src/contracts/graph.rs"),
    ];
    let needle = format!("{DOMAIN_CONTRACT_IMPORT_HEAD}{DOMAIN_CONTRACT_IMPORT_TAIL}");
    let mut offenders = Vec::new();

    for relative_root in ["src", "tests"] {
        collect_domain_contract_imports(
            manifest_dir.join(relative_root).as_path(),
            &allowed_reexports,
            needle.as_str(),
            &mut offenders,
        );
    }

    assert!(
        offenders.is_empty(),
        "Studio code should import Studio API contracts through crate::contracts or xiuxian_wendao_studio::contracts; only src/contracts transition modules may re-export the domain transition path:\n{}",
        offenders.join("\n")
    );
}
