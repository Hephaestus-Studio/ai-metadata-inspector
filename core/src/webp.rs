//! Low-level WebP container parsing, metadata extraction, and lossless sanitization.
//!
//! Complies with Google's WebP Container (RIFF-based) Specification, supporting
//! simple lossy (`VP8 `), simple lossless (`VP8L`), extended (`VP8X`), animation (`ANIM`/`ANMF`),
//! EXIF (`EXIF`), XMP (`XMP `), ICC profiles (`ICCP`), and Content Credentials (`c2pa`/`JUMB`).
//!
//! Handles automatic bitmask flag updating in `VP8X` chunks when metadata chunks are stripped.

use crate::types::{CleanOptions, CleanResult, RawChunkInfo};

/// Represents an individual parsed WebP RIFF chunk.
#[derive(Debug, Clone)]
pub struct WebpChunk<'a> {
    /// 4-character code (FourCC) identifier for the chunk (e.g. `b"VP8X"`, `b"EXIF"`, `b"XMP "`).
    pub fourcc: [u8; 4],
    /// Length of the chunk data payload in bytes (excluding FourCC and 4-byte size header).
    pub size: u32,
    /// Slice referencing the raw payload data (excluding 1-byte padding if odd).
    pub data: &'a [u8],
    /// Complete raw chunk slice including FourCC (4B), size (4B), payload (N B), and padding byte (if odd).
    pub full_slice: &'a [u8],
}

/// Structured summary of metadata extracted from a WebP image container.
#[derive(Debug, Clone, Default)]
pub struct WebpMetadataSummary {
    /// Image width in pixels extracted from `VP8X`, `VP8 `, or `VP8L` headers.
    pub width: u32,
    /// Image height in pixels extracted from `VP8X`, `VP8 `, or `VP8L` headers.
    pub height: u32,
    /// All decoded key-value text chunks and XMP packets.
    pub raw_chunks: Vec<RawChunkInfo>,
    /// Raw binary EXIF TIFF header payload extracted from the `EXIF` chunk, if present.
    pub raw_exif: Option<Vec<u8>>,
    /// Indicates whether an embedded ICC color profile (`ICCP`) was found.
    pub has_icc: bool,
    /// Indicates whether an XMP metadata packet (`XMP `) was found.
    pub has_xmp: bool,
    /// Indicates whether C2PA / Content Credentials provenance assertions were found.
    pub has_c2pa: bool,
    /// String representation of distinct FourCC chunk types discovered (e.g. `"VP8X"`, `"EXIF"`).
    pub found_chunk_types: Vec<String>,
}

/// Parses a byte slice into an ordered sequence of [`WebpChunk`] items.
///
/// # Errors
/// Returns an error if the RIFF/WEBP container signature is missing or if chunk lengths
/// exceed file boundaries.
pub fn parse_webp_chunks<'a>(bytes: &'a [u8]) -> Result<Vec<WebpChunk<'a>>, String> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return Err("Not a valid WebP file: missing RIFF/WEBP signature".to_string());
    }

    let mut chunks = Vec::new();
    let mut offset = 12;

    while offset + 8 <= bytes.len() {
        let chunk_start = offset;
        let fourcc: [u8; 4] = bytes[offset..offset + 4].try_into().unwrap();
        let size = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        offset += 8;

        if offset + size > bytes.len() {
            return Err("Corrupted WebP: chunk size extends beyond file boundary".to_string());
        }

        let data = &bytes[offset..offset + size];
        offset += size;

        // WebP chunks with odd payload sizes are padded with 1 null byte
        if size % 2 != 0 && offset < bytes.len() {
            offset += 1;
        }

        let full_slice = &bytes[chunk_start..offset];

        chunks.push(WebpChunk {
            fourcc,
            size: size as u32,
            data,
            full_slice,
        });
    }

    Ok(chunks)
}

/// Extracts dimensions, EXIF payload, XMP packets, ICC profile presence, and chunk identifiers from WebP bytes.
pub fn extract_webp_metadata(bytes: &[u8]) -> WebpMetadataSummary {
    let mut summary = WebpMetadataSummary::default();

    if let Ok(chunks) = parse_webp_chunks(bytes) {
        for chunk in chunks {
            let fourcc_str = String::from_utf8_lossy(&chunk.fourcc).to_string();
            if !summary.found_chunk_types.contains(&fourcc_str) {
                summary.found_chunk_types.push(fourcc_str);
            }

            match &chunk.fourcc {
                b"VP8X" => {
                    if chunk.data.len() >= 10 {
                        // 24-bit canvas width - 1 (bytes 4..7) and height - 1 (bytes 7..10)
                        let w = (chunk.data[4] as u32)
                            | ((chunk.data[5] as u32) << 8)
                            | ((chunk.data[6] as u32) << 16);
                        let h = (chunk.data[7] as u32)
                            | ((chunk.data[8] as u32) << 8)
                            | ((chunk.data[9] as u32) << 16);
                        summary.width = w + 1;
                        summary.height = h + 1;
                    }
                }
                b"VP8 " => {
                    if summary.width == 0 && chunk.data.len() >= 10 {
                        // Keyframe header (bytes 6-9 contain 14-bit dimensions)
                        let w = ((chunk.data[6] as u32) | ((chunk.data[7] as u32) << 8)) & 0x3FFF;
                        let h = ((chunk.data[8] as u32) | ((chunk.data[9] as u32) << 8)) & 0x3FFF;
                        summary.width = w;
                        summary.height = h;
                    }
                }
                b"VP8L" => {
                    if summary.width == 0 && chunk.data.len() >= 5 && chunk.data[0] == 0x2F {
                        // VP8L lossless header: 14 bits width-1, 14 bits height-1
                        let b1 = chunk.data[1] as u32;
                        let b2 = chunk.data[2] as u32;
                        let b3 = chunk.data[3] as u32;
                        let b4 = chunk.data[4] as u32;

                        let w = (b1 | ((b2 & 0x3F) << 8)) + 1;
                        let h = ((b2 >> 6) | (b3 << 2) | ((b4 & 0x0F) << 10)) + 1;
                        summary.width = w;
                        summary.height = h;
                    }
                }
                b"EXIF" => {
                    if chunk.data.starts_with(b"Exif\0\0") {
                        summary.raw_exif = Some(chunk.data[6..].to_vec());
                    } else {
                        summary.raw_exif = Some(chunk.data.to_vec());
                    }
                }
                b"XMP " => {
                    summary.has_xmp = true;
                    let xmp_str = String::from_utf8_lossy(chunk.data).to_string();
                    summary.raw_chunks.push(RawChunkInfo {
                        key: "XMP".to_string(),
                        value: xmp_str,
                        chunk_type: "XMP".to_string(),
                    });
                }
                b"ICCP" => {
                    summary.has_icc = true;
                }
                b"JUMB" | b"jumb" | b"c2pa" | b"caPI" => {
                    summary.has_c2pa = true;
                }
                _ => {}
            }
        }
    }

    summary
}

/// Strips selected metadata chunks from a WebP file losslessly and updates `VP8X` header flags.
///
/// Preserves image visual data (`VP8 `, `VP8L`, `ALPH`, `ANIM`, `ANMF`), updates
/// bitmask flags in `VP8X` header, and recalculates total RIFF file size header.
///
/// # Errors
/// Returns an error if the input bytes do not contain a valid WebP file.
pub fn clean_webp_lossless(bytes: &[u8], options: &CleanOptions) -> Result<CleanResult, String> {
    let chunks = parse_webp_chunks(bytes)?;
    let original_size = bytes.len();

    let mut kept_chunks = Vec::new();
    let mut stripped_exif = false;
    let mut stripped_xmp = false;
    let mut stripped_icc = false;

    for chunk in chunks {
        let fourcc = &chunk.fourcc;
        let mut keep = false;

        // Whitelist of visual, animation, and essential frame chunks
        match fourcc {
            b"VP8 " | b"VP8L" | b"VP8X" | b"ALPH" | b"ANIM" | b"ANMF" => {
                keep = true;
            }
            b"ICCP" => {
                if !options.strip_all && !options.strip_icc_profile {
                    keep = true;
                } else {
                    stripped_icc = true;
                }
            }
            b"EXIF" => {
                if !options.strip_all && !options.strip_exif && !options.strip_gps {
                    keep = true;
                } else {
                    stripped_exif = true;
                }
            }
            b"XMP " => {
                if !options.strip_all && !options.strip_xmp && !options.strip_ai_metadata {
                    keep = true;
                } else {
                    stripped_xmp = true;
                }
            }
            _ => {
                // Any other chunk (JUMB, jumb, c2pa, etc.)
                if !options.strip_all && !options.strip_c2pa {
                    keep = true;
                }
            }
        }

        if keep {
            kept_chunks.push(chunk);
        }
    }

    // Reconstruct WebP file
    let mut output = Vec::with_capacity(original_size);
    // Header placeholder: "RIFF" + 4 bytes size + "WEBP"
    output.extend_from_slice(b"RIFF\0\0\0\0WEBP");

    for chunk in kept_chunks {
        if &chunk.fourcc == b"VP8X" && chunk.data.len() >= 10 {
            // Update flags in VP8X chunk (Bit 2: XMP, Bit 3: EXIF, Bit 5: ICCP)
            let mut flags = chunk.data[0];
            if stripped_xmp {
                flags &= !(1 << 2);
            }
            if stripped_exif {
                flags &= !(1 << 3);
            }
            if stripped_icc {
                flags &= !(1 << 5);
            }

            output.extend_from_slice(b"VP8X");
            output.extend_from_slice(&(chunk.size as u32).to_le_bytes());
            output.push(flags);
            output.extend_from_slice(&chunk.data[1..]);
            if chunk.size % 2 != 0 {
                output.push(0);
            }
        } else {
            output.extend_from_slice(chunk.full_slice);
        }
    }

    // Update RIFF payload size (total file size - 8 bytes)
    let riff_size = (output.len().saturating_sub(8)) as u32;
    output[4..8].copy_from_slice(&riff_size.to_le_bytes());

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

    // Construct a minimal valid dummy WebP byte sequence with VP8X for testing
    fn create_dummy_webp(include_exif: bool, include_xmp: bool) -> Vec<u8> {
        let mut webp = Vec::new();
        // RIFF header placeholder
        webp.extend_from_slice(b"RIFF\0\0\0\0WEBP");

        // Flags: bit 3 (EXIF = 0x08), bit 2 (XMP = 0x04)
        let mut flags: u8 = 0;
        if include_exif {
            flags |= 1 << 3;
        }
        if include_xmp {
            flags |= 1 << 2;
        }

        // VP8X payload: flags (1B), reserved (3B), canvas width-1 (3B), canvas height-1 (3B)
        // 1x1 image -> width-1: 0, height-1: 0
        let vp8x_data = [flags, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        append_chunk(&mut webp, b"VP8X", &vp8x_data);

        if include_exif {
            append_chunk(&mut webp, b"EXIF", b"Exif\0\0dummy_exif_payload");
        }

        if include_xmp {
            append_chunk(&mut webp, b"XMP ", b"<x:xmpmeta>dummy_xmp</x:xmpmeta>");
        }

        // Dummy VP8 image data (odd length to test padding)
        append_chunk(&mut webp, b"VP8 ", &[0x01, 0x02, 0x03]);

        // Fix RIFF size
        let riff_size = (webp.len() - 8) as u32;
        webp[4..8].copy_from_slice(&riff_size.to_le_bytes());

        webp
    }

    fn append_chunk(buffer: &mut Vec<u8>, fourcc: &[u8; 4], data: &[u8]) {
        buffer.extend_from_slice(fourcc);
        let size = data.len() as u32;
        buffer.extend_from_slice(&size.to_le_bytes());
        buffer.extend_from_slice(data);
        if size % 2 != 0 {
            buffer.push(0); // 1-byte padding
        }
    }

    #[test]
    fn test_parse_webp_chunks() {
        let webp = create_dummy_webp(true, true);
        let chunks = parse_webp_chunks(&webp).expect("Valid WebP should parse");
        assert_eq!(chunks.len(), 4); // VP8X, EXIF, XMP , VP8
        assert_eq!(&chunks[0].fourcc, b"VP8X");
        assert_eq!(&chunks[1].fourcc, b"EXIF");
        assert_eq!(&chunks[2].fourcc, b"XMP ");
        assert_eq!(&chunks[3].fourcc, b"VP8 ");
    }

    #[test]
    fn test_extract_webp_metadata() {
        let webp = create_dummy_webp(true, true);
        let summary = extract_webp_metadata(&webp);
        assert_eq!(summary.width, 1);
        assert_eq!(summary.height, 1);
        assert!(summary.raw_exif.is_some());
        assert!(summary.has_xmp);
        assert_eq!(summary.raw_chunks.len(), 1);
        assert_eq!(summary.raw_chunks[0].key, "XMP");
    }

    #[test]
    fn test_clean_webp_lossless() {
        let webp = create_dummy_webp(true, true);
        let options = CleanOptions {
            strip_all: true,
            ..Default::default()
        };

        let result = clean_webp_lossless(&webp, &options).expect("Clean should succeed");
        assert!(result.success);
        assert!(result.cleaned_size < result.original_size);
        assert!(result.bytes_removed > 0);

        let cleaned_summary = extract_webp_metadata(&result.cleaned_bytes);
        assert!(cleaned_summary.raw_exif.is_none());
        assert!(!cleaned_summary.has_xmp);
        assert_eq!(cleaned_summary.raw_chunks.len(), 0);
        assert_eq!(cleaned_summary.width, 1);
        assert_eq!(cleaned_summary.height, 1);

        // Verify VP8X flags were cleared
        let cleaned_chunks = parse_webp_chunks(&result.cleaned_bytes).unwrap();
        let vp8x_flags = cleaned_chunks[0].data[0];
        assert_eq!(vp8x_flags & (1 << 3), 0); // EXIF flag cleared
        assert_eq!(vp8x_flags & (1 << 2), 0); // XMP flag cleared
    }
}
