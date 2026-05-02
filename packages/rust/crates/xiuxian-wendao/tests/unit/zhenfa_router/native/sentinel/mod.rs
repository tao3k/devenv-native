mod analysis;
mod drift;
mod observation;
mod scope;

pub(super) use super::{
    AffectedDoc, DriftConfidence, ObservationBus, ObservationRef, ObservationSignal, Path,
    SemanticDriftSignal, compute_file_hash, extract_pattern_symbols, is_high_noise_file,
    is_ignorable_path, is_source_code, matches_scope_filter, mpsc, signals_to_status_batch,
    to_pascal_case, verify_file_stable,
};
