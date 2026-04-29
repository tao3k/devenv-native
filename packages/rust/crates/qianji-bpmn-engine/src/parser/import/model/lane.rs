#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawLaneMembershipSpec {
    pub(crate) set_id: Option<String>,
    pub(crate) set_name: Option<String>,
    pub(crate) id: Option<String>,
    pub(crate) name: Option<String>,
}
