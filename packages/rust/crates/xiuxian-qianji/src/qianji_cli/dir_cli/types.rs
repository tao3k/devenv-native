use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DirCliCommand {
    Show { target: ShowCliTarget },
    Check { dir: PathBuf },
    Advance { dir: PathBuf, to: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ShowCliTarget {
    Dir(PathBuf),
    Graph(PathBuf),
    Contract(String),
    AnchoredScenario {
        anchor: PathBuf,
        scenario: String,
        dir: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DirCliOutput {
    pub(crate) rendered: String,
    pub(crate) exit_code: i32,
}
