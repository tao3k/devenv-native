use std::path::Path;

use super::LegacyOfficeFormat;

pub(crate) fn legacy_office_markdown(
    path: &Path,
    format: LegacyOfficeFormat,
    text: &str,
) -> Result<String, String> {
    let file_name = path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or_else(|| format!("legacy Office source `{}` has no file name", path.display()))?;
    match format {
        LegacyOfficeFormat::Xls => Ok(format!(
            "# {file_name}\n\n{}\n\n```tsv\n{text}\n```\n",
            format.label()
        )),
        LegacyOfficeFormat::Doc | LegacyOfficeFormat::Ppt => {
            Ok(format!("# {file_name}\n\n{}\n\n{text}\n", format.label()))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{LegacyOfficeFormat, legacy_office_markdown};

    #[test]
    fn xls_markdown_preserves_tabular_boundaries() {
        let markdown = match legacy_office_markdown(
            Path::new("rates.xls"),
            LegacyOfficeFormat::Xls,
            "name\tvalue\nalpha\t42",
        ) {
            Ok(markdown) => markdown,
            Err(error) => panic!("markdown failed: {error}"),
        };

        assert!(markdown.contains("```tsv\nname\tvalue\nalpha\t42\n```"));
    }
}
