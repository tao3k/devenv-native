//! Internal TSV readers for episteme source-contract tables.

use super::EpistemeSourceContractParseError;

pub(super) fn read_tsv(
    raw: &str,
    expected_header: &[&'static str],
) -> Result<Vec<Vec<String>>, EpistemeSourceContractParseError> {
    let mut lines = raw.lines();
    let header = lines
        .next()
        .unwrap_or_default()
        .split('\t')
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let expected = expected_header
        .iter()
        .map(|field| (*field).to_string())
        .collect::<Vec<_>>();
    if header != expected {
        return Err(EpistemeSourceContractParseError::TsvHeader {
            expected,
            actual: header,
        });
    }

    lines
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            let fields = line
                .trim_end_matches('\r')
                .split('\t')
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();
            if fields.len() != expected_header.len() {
                return Err(EpistemeSourceContractParseError::TsvRowWidth {
                    row: index + 2,
                    expected: expected_header.len(),
                    actual: fields.len(),
                });
            }
            Ok(fields)
        })
        .collect()
}

pub(super) fn parse_number<T>(
    row: usize,
    field: &'static str,
    value: &str,
) -> Result<T, EpistemeSourceContractParseError>
where
    T: std::str::FromStr,
{
    value
        .parse::<T>()
        .map_err(|_| EpistemeSourceContractParseError::InvalidNumber {
            row,
            field,
            value: value.to_string(),
        })
}
