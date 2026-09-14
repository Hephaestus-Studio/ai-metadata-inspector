//! Low-level JPEG segment parsing, metadata extraction, and lossless sanitization.
//!
//! Complies with the ITU-T T.81 / ISO/IEC 10918-1 JPEG standard and EXIF/XMP/C2PA specifications,
//! supporting segment identification (`APP0`–`APP15`, `SOF0`–`SOF15`, `COM`, `DQT`, `DHT`, `DRI`, `SOS`),
//! metadata extraction, and lossless stripping without altering compressed DCT scan entropy.

use crate::types::{CleanOptions, CleanResult, RawChunkInfo};

/// Standard 2-byte JPEG Start of Image marker (`0xFF 0xD8`).
pub const JPEG_SOI: &[u8; 2] = &[0xFF, 0xD8];

/// Standard 2-byte JPEG End of Image marker (`0xFF 0xD9`).
pub const JPEG_EOI: &[u8; 2] = &[0xFF, 0xD9];

/// Represents an individual parsed JPEG marker segment with slice boundaries.
#[derive(Debug, Clone)]
pub struct JpegSegment<'a> {
    /// Second byte of the marker code (e.g. `0xE1` for `APP1`, `0xC0` for `SOF0`, `0xFE` for `COM`).
    pub marker: u8,
    /// Slice referencing the segment payload data (excluding marker and length bytes).
    pub data: &'a [u8],
    /// Complete raw segment slice including marker prefix (`0xFF <marker>`), 2-byte length, and payload.
    pub full_slice: &'a [u8],
}

/// Structured summary of metadata extracted from a JPEG image container.
#[derive(Debug, Clone, Default)]
pub struct JpegMetadataSummary {
    /// Image width in pixels extracted from SOF (Start of Frame) marker.
    pub width: u32,
    /// Image height in pixels extracted from SOF (Start of Frame) marker.
    pub height: u32,
    /// All decoded key-value text chunks, XMP strings, and comments.
    pub raw_chunks: Vec<RawChunkInfo>,
    /// Raw binary EXIF TIFF header payload extracted from `APP1` (`Exif\0\0`), if present.
    pub raw_exif: Option<Vec<u8>>,
    /// Indicates whether an embedded ICC color profile (`APP2` / `ICC_PROFILE\0`) was found.
    pub has_icc: bool,
    /// Indicates whether an XMP metadata packet (`APP1` / `http://ns.adobe.com/xap/1.0/\0`) was found.
    pub has_xmp: bool,
    /// Indicates whether C2PA / Content Credentials provenance assertions (`APP11`) were found.
    pub has_c2pa: bool,
    /// Hex-formatted string representations of distinct markers discovered (e.g. `"0xE1"`, `"0xC0"`).
    pub found_markers: Vec<String>,
}

/// Parses a JPEG byte slice into an ordered sequence of header [`JpegSegment`] items
/// and isolates the trailing compressed entropy scan slice starting from `SOS` (`0xDA`).
///
/// # Errors
/// Returns an error if the JPEG SOI header is missing or segment length fields exceed slice boundaries.
pub fn parse_jpeg_segments<'a>(
    bytes: &'a [u8],
) -> Result<(Vec<JpegSegment<'a>>, &'a [u8]), String> {
    if bytes.len() < 4 || &bytes[0..2] != JPEG_SOI {
        return Err("Not a valid JPEG file: missing SOI marker".to_string());
    }

    let mut segments = Vec::new();
    let mut offset = 2;

    while offset + 2 <= bytes.len() {
        if bytes[offset] != 0xFF {
            // Skip filler or alignment bytes preceding marker prefix
            offset += 1;
            continue;
        }

        let marker = bytes[offset + 1];
        if marker == 0x00 || marker == 0xFF {
            // Stuffed byte in scan stream or repeated padding prefix
            offset += 1;
            continue;
        }

        // Check for SOS (Start of Scan - 0xDA)
        if marker == 0xDA {
            // From SOS onward, remainder of the stream contains entropy-coded scan data until EOI
            return Ok((segments, &bytes[offset..]));
        }

        if marker == 0xD9 {
            // End of Image marker
            break;
        }

        // Standalone markers without length fields (RST0..RST7, SOI, TEM)
        if (0xD0..=0xD7).contains(&marker) || marker == 0x01 {
            offset += 2;
            continue;
        }

        // Marker with 2-byte big-endian payload length
        if offset + 4 > bytes.len() {
            break;
        }

        let length = u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]) as usize;
        if length < 2 {
            return Err("Invalid JPEG segment: length less than 2 bytes".to_string());
        }

        let seg_end = offset + 2 + length;
        if seg_end > bytes.len() {
            return Err("Corrupted JPEG: segment extends beyond file boundary".to_string());
        }

        let full_slice = &bytes[offset..seg_end];
        let data = &bytes[offset + 4..seg_end]; // skip 0xFF, marker, and 2 length bytes

        segments.push(JpegSegment {
            marker,
            data,
            full_slice,
        });

        offset = seg_end;
    }

    Ok((segments, &bytes[offset..]))
}

/// Extracts dimensions, EXIF payload, XMP packets, ICC profile presence, and comments from JPEG bytes.
pub fn extract_jpeg_metadata(bytes: &[u8]) -> JpegMetadataSummary {
    let mut summary = JpegMetadataSummary::default();

    if let Ok((segments, _)) = parse_jpeg_segments(bytes) {
        for seg in segments {
            let marker_str = format!("0x{:02X}", seg.marker);
            if !summary.found_markers.contains(&marker_str) {
                summary.found_markers.push(marker_str);
            }

            // Extract dimensions from SOF markers (SOF0..SOF3, SOF5..SOF7, SOF9..SOF11)
            if (0xC0..=0xC3).contains(&seg.marker)
                || (0xC5..=0xC7).contains(&seg.marker)
                || (0xC9..=0xCB).contains(&seg.marker)
            {
                if seg.data.len() >= 5 {
                    // byte 0: sample precision, bytes 1-2: height, bytes 3-4: width
                    summary.height = u16::from_be_bytes([seg.data[1], seg.data[2]]) as u32;
                    summary.width = u16::from_be_bytes([seg.data[3], seg.data[4]]) as u32;
                }
            }

            // APP1 (0xE1) - EXIF or XMP
            if seg.marker == 0xE1 {
                if seg.data.starts_with(b"Exif\0\0") {
                    summary.raw_exif = Some(seg.data[6..].to_vec());
                } else if seg.data.starts_with(b"http://ns.adobe.com/xap/1.0/\0") {
                    summary.has_xmp = true;
                    let xmp_str = String::from_utf8_lossy(&seg.data[29..]).to_string();
                    summary.raw_chunks.push(RawChunkInfo {
                        key: "XMP".to_string(),
                        value: xmp_str,
                        chunk_type: "APP1 (XMP)".to_string(),
                    });
                }
            }

            // APP2 (0xE2) - ICC Profile
            if seg.marker == 0xE2 && seg.data.starts_with(b"ICC_PROFILE\0") {
                summary.has_icc = true;
            }

            // APP11 (0xEB) - C2PA / Content Credentials / JUMBF
            if seg.marker == 0xEB || seg.data.starts_with(b"JP\0") || seg.data.starts_with(b"c2pa")
            {
                summary.has_c2pa = true;
            }

            // COM (0xFE) - Plain text comments
            if seg.marker == 0xFE {
                let comment = String::from_utf8_lossy(seg.data).to_string();
                summary.raw_chunks.push(RawChunkInfo {
                    key: "Comment".to_string(),
                    value: comment,
                    chunk_type: "COM".to_string(),
                });
            }
        }
    }

    summary
}

/// Strips selected metadata markers from a JPEG file losslessly without decompressing DCT scan data.
///
/// Preserves frame headers (`SOFn`), quantization tables (`DQT`), Huffman tables (`DHT`),
/// restart markers (`DRI`), color profiles (`APP2`), and trailing entropy data (`SOS` through `EOI`).
///
/// # Errors
/// Returns an error if the input bytes do not contain a valid JPEG SOI header.
pub fn clean_jpeg_lossless(bytes: &[u8], options: &CleanOptions) -> Result<CleanResult, String> {
    if bytes.len() < 4 || &bytes[0..2] != JPEG_SOI {
        return Err("Not a valid JPEG file: missing SOI marker".to_string());
    }

    let original_size = bytes.len();
    let mut output = Vec::with_capacity(original_size);
    output.extend_from_slice(JPEG_SOI);

    let (segments, scan_data) = parse_jpeg_segments(bytes)?;

    for seg in segments {
        let mut keep = true;
        let marker = seg.marker;

        if options.strip_all {
            match marker {
                // Strip APP1 (EXIF/XMP), APP3..APP13 (C2PA/IPTC/Vendor), APP15, COM (0xFE)
                0xE1 | 0xE3..=0xED | 0xEF | 0xFE => {
                    keep = false;
                }
                0xE2 => {
                    // APP2 ICC Profile
                    if options.strip_icc_profile {
                        keep = false;
                    }
                }
                _ => {}
            }
        } else {
            match marker {
                0xE1 => {
                    if seg.data.starts_with(b"Exif\0\0") {
                        if options.strip_exif || options.strip_gps {
                            keep = false;
                        }
                    } else if seg.data.starts_with(b"http://ns.adobe.com/xap/1.0/\0") {
                        if options.strip_xmp || options.strip_ai_metadata {
                            keep = false;
                        }
                    } else if options.strip_all {
                        keep = false;
                    }
                }
                0xEB => {
                    if options.strip_c2pa {
                        keep = false;
                    }
                }
                0xED => {
                    if options.strip_exif || options.strip_all {
                        keep = false;
                    }
                }
                0xFE => {
                    if options.strip_comments || options.strip_ai_metadata {
                        keep = false;
                    }
                }
                0xE2 => {
                    if options.strip_icc_profile {
                        keep = false;
                    }
                }
                0xE3..=0xEA | 0xEC | 0xEF => {
                    if options.strip_all || options.strip_comments {
                        keep = false;
                    }
                }
                _ => {}
            }
        }

        if keep {
            output.extend_from_slice(seg.full_slice);
        }
    }

    // Append the untouched SOS segment and entropy bitstream
    if !scan_data.is_empty() {
        output.extend_from_slice(scan_data);
    } else {
        output.extend_from_slice(JPEG_EOI);
    }

    let cleaned_size = output.len();
    let bytes_removed = original_size.saturating_sub(cleaned_size);
    let percentage_reduced = if original_size > 0 {
        (bytes_removed as f64 / original_size as f64) * 100.0
    } else {
        0.0
    };

    Ok(CleanResult {
        success: true,
        original_size,
        cleaned_size,
        bytes_removed,
        percentage_reduced,
        cleaned_bytes: output,
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Construct a minimal valid dummy JPEG byte sequence for unit testing
    fn create_dummy_jpeg(include_exif: bool, include_com: bool) -> Vec<u8> {
        let mut jpeg = Vec::new();
        // SOI
        jpeg.extend_from_slice(JPEG_SOI);

        if include_exif {
            // APP1 Exif segment
            let mut exif_payload = Vec::new();
            exif_payload.extend_from_slice(b"Exif\0\0dummy_exif_data");
            append_segment(&mut jpeg, 0xE1, &exif_payload);
        }

        if include_com {
            // COM segment
            append_segment(&mut jpeg, 0xFE, b"prompt: A futuristic cyberpunk city");
        }

        // SOF0 segment: 1x1 image (precision: 8, height: 1, width: 1, components: 1)
        let sof0_payload = [0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00];
        append_segment(&mut jpeg, 0xC0, &sof0_payload);

        // SOS marker and dummy entropy scan data + EOI
        jpeg.extend_from_slice(&[0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3F, 0x00]);
        jpeg.extend_from_slice(&[0x12, 0x34, 0x56]); // scan data
        jpeg.extend_from_slice(JPEG_EOI);

        jpeg
    }

    fn append_segment(buffer: &mut Vec<u8>, marker: u8, data: &[u8]) {
        buffer.extend_from_slice(&[0xFF, marker]);
        let length = (data.len() + 2) as u16;
        buffer.extend_from_slice(&length.to_be_bytes());
        buffer.extend_from_slice(data);
    }

    #[test]
    fn test_parse_jpeg_segments() {
        let jpeg = create_dummy_jpeg(true, true);
        let (segments, scan_data) = parse_jpeg_segments(&jpeg).expect("Valid JPEG should parse");
        assert_eq!(segments.len(), 3); // APP1, COM, SOF0
        assert_eq!(segments[0].marker, 0xE1);
        assert_eq!(segments[1].marker, 0xFE);
        assert_eq!(segments[2].marker, 0xC0);
        assert!(scan_data.starts_with(&[0xFF, 0xDA]));
    }

    #[test]
    fn test_extract_jpeg_metadata() {
        let jpeg = create_dummy_jpeg(true, true);
        let summary = extract_jpeg_metadata(&jpeg);
        assert_eq!(summary.width, 1);
        assert_eq!(summary.height, 1);
        assert!(summary.raw_exif.is_some());
        assert_eq!(summary.raw_chunks.len(), 1);
        assert_eq!(summary.raw_chunks[0].key, "Comment");
        assert_eq!(
            summary.raw_chunks[0].value,
            "prompt: A futuristic cyberpunk city"
        );
    }

    #[test]
    fn test_clean_jpeg_lossless() {
        let jpeg = create_dummy_jpeg(true, true);
        let options = CleanOptions {
            strip_all: true,
            ..Default::default()
        };

        let result = clean_jpeg_lossless(&jpeg, &options).expect("Clean should succeed");
        assert!(result.success);
        assert!(result.cleaned_size < result.original_size);
        assert!(result.bytes_removed > 0);

        let cleaned_summary = extract_jpeg_metadata(&result.cleaned_bytes);
        assert!(cleaned_summary.raw_exif.is_none());
        assert_eq!(cleaned_summary.raw_chunks.len(), 0);
        assert_eq!(cleaned_summary.width, 1);
        assert_eq!(cleaned_summary.height, 1);
    }
}
