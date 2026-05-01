use std::fmt::{self, Write as _};

pub(super) fn push_xml(xml: &mut String, args: fmt::Arguments<'_>) {
    let _ = xml.write_fmt(args);
}
