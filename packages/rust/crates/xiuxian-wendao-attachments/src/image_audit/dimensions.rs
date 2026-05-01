pub(super) fn parse_dimensions(format: &str, bytes: &[u8]) -> Option<(u32, u32, &'static str)> {
    match format {
        "png" => png_dimensions(bytes).map(|(width, height)| (width, height, "png_ihdr")),
        "jpeg" => jpeg_dimensions(bytes).map(|(width, height)| (width, height, "jpeg_sof")),
        "tiff" => tiff_dimensions(bytes).map(|(width, height)| (width, height, "tiff_ifd")),
        "bmp" => bmp_dimensions(bytes).map(|(width, height)| (width, height, "bmp_dib")),
        "webp" => webp_dimensions(bytes).map(|(width, height)| (width, height, "webp_header")),
        "gif" => gif_dimensions(bytes).map(|(width, height)| (width, height, "gif_lsd")),
        _ => None,
    }
}

fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 24 || &bytes[0..8] != PNG_SIGNATURE || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    non_zero_dimensions(width, height)
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 4 || bytes[0] != 0xff || bytes[1] != 0xd8 {
        return None;
    }
    let mut index = 2usize;
    while index + 3 < bytes.len() {
        while index < bytes.len() && bytes[index] != 0xff {
            index += 1;
        }
        while index < bytes.len() && bytes[index] == 0xff {
            index += 1;
        }
        if index >= bytes.len() {
            return None;
        }
        let marker = bytes[index];
        index += 1;
        if marker == 0xd9 || marker == 0xda {
            return None;
        }
        if marker == 0x01 || (0xd0..=0xd7).contains(&marker) {
            continue;
        }
        if index + 1 >= bytes.len() {
            return None;
        }
        let segment_len = usize::from(u16::from_be_bytes([bytes[index], bytes[index + 1]]));
        if segment_len < 2 {
            return None;
        }
        let segment_start = index + 2;
        let segment_end = index + segment_len;
        if segment_end > bytes.len() {
            return None;
        }
        if is_jpeg_start_of_frame(marker) && segment_start + 4 < segment_end {
            let height = u16::from_be_bytes([bytes[segment_start + 1], bytes[segment_start + 2]]);
            let width = u16::from_be_bytes([bytes[segment_start + 3], bytes[segment_start + 4]]);
            return non_zero_dimensions(u32::from(width), u32::from(height));
        }
        index = segment_end;
    }
    None
}

fn is_jpeg_start_of_frame(marker: u8) -> bool {
    matches!(
        marker,
        0xc0 | 0xc1 | 0xc2 | 0xc3 | 0xc5 | 0xc6 | 0xc7 | 0xc9 | 0xca | 0xcb | 0xcd | 0xce | 0xcf
    )
}

fn tiff_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let endian = TiffEndian::from_header(bytes)?;
    let ifd_offset = usize::try_from(endian.read_u32(bytes.get(4..8)?)?).ok()?;
    let entry_count = usize::from(endian.read_u16(bytes.get(ifd_offset..ifd_offset + 2)?)?);
    let entries_start = ifd_offset.checked_add(2)?;
    let mut width = None;
    let mut height = None;
    for entry_index in 0..entry_count {
        let offset = entries_start.checked_add(entry_index.checked_mul(12)?)?;
        let entry = bytes.get(offset..offset + 12)?;
        match endian.read_u16(entry.get(0..2)?)? {
            256 => width = tiff_dimension_value(entry, endian),
            257 => height = tiff_dimension_value(entry, endian),
            _ => {}
        }
        if width.is_some() && height.is_some() {
            break;
        }
    }
    non_zero_dimensions(width?, height?)
}

#[derive(Debug, Clone, Copy)]
enum TiffEndian {
    Little,
    Big,
}

impl TiffEndian {
    fn from_header(bytes: &[u8]) -> Option<Self> {
        let endian = match bytes.get(0..2)? {
            b"II" => Self::Little,
            b"MM" => Self::Big,
            _ => return None,
        };
        (endian.read_u16(bytes.get(2..4)?)? == 42).then_some(endian)
    }

    fn read_u16(self, bytes: &[u8]) -> Option<u16> {
        let array = bytes.try_into().ok()?;
        Some(match self {
            Self::Little => u16::from_le_bytes(array),
            Self::Big => u16::from_be_bytes(array),
        })
    }

    fn read_u32(self, bytes: &[u8]) -> Option<u32> {
        let array = bytes.try_into().ok()?;
        Some(match self {
            Self::Little => u32::from_le_bytes(array),
            Self::Big => u32::from_be_bytes(array),
        })
    }
}

fn tiff_dimension_value(entry: &[u8], endian: TiffEndian) -> Option<u32> {
    let value_type = endian.read_u16(entry.get(2..4)?)?;
    let value_count = endian.read_u32(entry.get(4..8)?)?;
    if value_count != 1 {
        return None;
    }
    match value_type {
        3 => Some(u32::from(endian.read_u16(entry.get(8..10)?)?)),
        4 => endian.read_u32(entry.get(8..12)?),
        _ => None,
    }
}

fn bmp_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 26 || &bytes[0..2] != b"BM" {
        return None;
    }
    let width = i32::from_le_bytes(bytes[18..22].try_into().ok()?);
    let height = i32::from_le_bytes(bytes[22..26].try_into().ok()?);
    if width <= 0 || height == 0 {
        return None;
    }
    non_zero_dimensions(u32::try_from(width).ok()?, height.unsigned_abs())
}

fn webp_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 20 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return None;
    }
    let mut index = 12usize;
    while index + 8 <= bytes.len() {
        let chunk_type = bytes.get(index..index + 4)?;
        let chunk_size = usize::try_from(u32::from_le_bytes(
            bytes.get(index + 4..index + 8)?.try_into().ok()?,
        ))
        .ok()?;
        let data_start = index.checked_add(8)?;
        let data_end = data_start.checked_add(chunk_size)?;
        let chunk = bytes.get(data_start..data_end)?;
        let dimensions = match chunk_type {
            b"VP8X" => webp_vp8x_dimensions(chunk),
            b"VP8L" => webp_vp8l_dimensions(chunk),
            b"VP8 " => webp_vp8_dimensions(chunk),
            _ => None,
        };
        if dimensions.is_some() {
            return dimensions;
        }
        let padded_size = chunk_size.checked_add(chunk_size % 2)?;
        index = data_start.checked_add(padded_size)?;
    }
    None
}

fn webp_vp8x_dimensions(chunk: &[u8]) -> Option<(u32, u32)> {
    if chunk.len() < 10 {
        return None;
    }
    let width = read_u24_le(chunk.get(4..7)?)?.checked_add(1)?;
    let height = read_u24_le(chunk.get(7..10)?)?.checked_add(1)?;
    non_zero_dimensions(width, height)
}

fn webp_vp8l_dimensions(chunk: &[u8]) -> Option<(u32, u32)> {
    if chunk.len() < 5 || chunk[0] != 0x2f {
        return None;
    }
    let width = u32::from(chunk[1]) | (u32::from(chunk[2] & 0x3f) << 8);
    let height = (u32::from(chunk[2] & 0xc0) >> 6)
        | (u32::from(chunk[3]) << 2)
        | (u32::from(chunk[4] & 0x0f) << 10);
    non_zero_dimensions(width.checked_add(1)?, height.checked_add(1)?)
}

fn webp_vp8_dimensions(chunk: &[u8]) -> Option<(u32, u32)> {
    if chunk.len() < 10 || chunk.get(3..6)? != b"\x9d\x01\x2a" {
        return None;
    }
    let width = u16::from_le_bytes(chunk.get(6..8)?.try_into().ok()?) & 0x3fff;
    let height = u16::from_le_bytes(chunk.get(8..10)?.try_into().ok()?) & 0x3fff;
    non_zero_dimensions(u32::from(width), u32::from(height))
}

fn read_u24_le(bytes: &[u8]) -> Option<u32> {
    (bytes.len() == 3)
        .then(|| u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[2]) << 16))
}

fn gif_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 10 || (bytes.get(0..6)? != b"GIF87a" && bytes.get(0..6)? != b"GIF89a") {
        return None;
    }
    let width = u16::from_le_bytes(bytes.get(6..8)?.try_into().ok()?);
    let height = u16::from_le_bytes(bytes.get(8..10)?.try_into().ok()?);
    non_zero_dimensions(u32::from(width), u32::from(height))
}

fn non_zero_dimensions(width: u32, height: u32) -> Option<(u32, u32)> {
    (width > 0 && height > 0).then_some((width, height))
}
