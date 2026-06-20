use xiuxian_config_core::ProjectDirs;
use xiuxian_git_repo::{
    RepoRefreshPolicy, RepoSpec, managed_checkout_root_for, managed_mirror_root_for,
};

#[test]
fn managed_repo_paths_follow_ghq_layout_for_remote_urls() {
    let spec = RepoSpec {
        id: "sciml".to_string(),
        local_path: None,
        remote_url: Some("https://github.com/SciML/BaseModelica.jl.git".to_string()),
        revision: None,
        refresh: RepoRefreshPolicy::Manual,
    };

    assert_eq!(
        managed_checkout_root_for(&spec),
        ProjectDirs::data_home()
            .join("xiuxian-wendao")
            .join("repo-intelligence")
            .join("repos")
            .join("github.com")
            .join("SciML")
            .join("BaseModelica.jl")
    );
    assert_eq!(
        managed_mirror_root_for(&spec),
        ProjectDirs::data_home()
            .join("xiuxian-wendao")
            .join("repo-intelligence")
            .join("mirrors")
            .join("github.com")
            .join("SciML")
            .join("BaseModelica.jl.git")
    );
}

#[test]
fn managed_repo_paths_support_scp_style_remote_urls() {
    let spec = RepoSpec {
        id: "sciml".to_string(),
        local_path: None,
        remote_url: Some("git@github.com:SciML/BaseModelica.jl.git".to_string()),
        revision: None,
        refresh: RepoRefreshPolicy::Manual,
    };

    assert_eq!(
        managed_checkout_root_for(&spec),
        ProjectDirs::data_home()
            .join("xiuxian-wendao")
            .join("repo-intelligence")
            .join("repos")
            .join("github.com")
            .join("SciML")
            .join("BaseModelica.jl")
    );
}
