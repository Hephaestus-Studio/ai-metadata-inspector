//! Low-level PNG chunk parsing, metadata extraction, and lossless sanitization.
//!
//! Complies with the W3C PNG (Portable Network Graphics) Specification (Second Edition)
//! and ISO/IEC 15948:2004, supporting standard critical/ancillary chunks, APNG animation
//! chunks, text chunks (`tEXt`, `zTXt`, `iTXt`), EXIF (`eXIf`), ICC profiles (`iCCP`),
//! and Content Credentials / C2PA assertions (`jumb`, `caPI`).

use crate::types::{CleanOptions, CleanResult, RawChunkInfo};
use flate2::read::ZlibDecoder;
use std::io::Read;

/// Standard 8-byte PNG file magic header signature.
pub const PNG_MAGIC: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

/// Represents a parsed PNG chunk with its slice boundaries and properties.
#[derive(Debug, Clone)]
pub struct PngChunk<'a> {
    /// Length of the chunk data field in bytes (excluding length, type, and CRC).
    pub length: u32,
    /// 4-byte chunk type identifier (e.g., `b"IHDR"`, `b"tEXt"`, `b"IDAT"`).
    pub chunk_type: [u8; 4],
    /// Slice referencing the chunk data payload.
    pub data: &'a [u8],
    /// 32-bit Cyclic Redundancy Check (CRC) value calculated over type and data.
    pub crc: u32,
    /// Complete raw chunk slice including length (4B), type (4B), data (N B), and CRC (4B).
    pub raw_slice: &'a [u8],
}

/// Structured summary of metadata extracted from a PNG container.
#[derive(Debug, Clone, Default)]
pub struct PngMetadataSummary {
    /// Image width in pixels extracted from the `IHDR` chunk.
    pub width: u32,
    /// Image height in pixels extracted from the `IHDR` chunk.
    pub height: u32,
    /// All decoded key-value text chunks (`tEXt`, `zTXt`, `iTXt`).
    pub raw_chunks: Vec<RawChunkInfo>,
    /// Raw binary EXIF payload extracted from the `eXIf` chunk, if present.
    pub raw_exif: Option<Vec<u8>>,
    /// Indicates whether an embedded ICC color profile (`iCCP`) was found.
    pub has_icc: bool,
    /// Indicates whether an XMP metadata packet (`XML:com.adobe.xmp`) was found.
    pub has_xmp: bool,
    /// Indicates whether C2PA / Content Credentials assertions were found.
    pub has_c2pa: bool,
    /// Distinct chunk type identifiers discovered during container scan.
    pub found_chunk_types: Vec<String>,
}

/// Parses a byte slice into an ordered sequence of [`PngChunk`] items.
///
/// # Errors
/// Returns an error if the PNG magic signature is invalid or if chunk length markers
/// exceed the input slice boundaries.
pub fn parse_png_chunks<'a>(bytes: &'a [u8]) -> Result<Vec<PngChunk<'a>>, String> {
    if bytes.len() < 8 || &bytes[0..8] != PNG_MAGIC {
        return Err("Not a valid PNG file: invalid magic signature".to_string());
    }

    let mut chunks = Vec::new();
    let mut offset = 8;

    while offset + 12 <= bytes.len() {
        let chunk_start = offset;
        let length = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        let chunk_type: [u8; 4] = bytes[offset + 4..offset + 8].try_into().unwrap();
        offset += 8;

        if offset + length + 4 > bytes.len() {
            return Err("Corrupted PNG: chunk length exceeds file bounds".to_string());
        }

        let data = &bytes[offset..offset + length];
        offset += length;

        let crc = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        let raw_slice = &bytes[chunk_start..offset];

        chunks.push(PngChunk {
            length: length as u32,
            chunk_type,
            data,
            crc,
            raw_slice,
        });

        if &chunk_type == b"IEND" {
            break;
        }
    }

    Ok(chunks)
}

/// Extracts dimensions, text metadata, EXIF, ICC profiles, and C2PA markers from PNG bytes.
///
/// Decodes both uncompressed (`tEXt`, `iTXt`) and zlib-compressed (`zTXt`, compressed `iTXt`)
/// metadata chunks safely without modifying image pixel streams.
pub fn extract_png_metadata(bytes: &[u8]) -> PngMetadataSummary {
    let mut summary = PngMetadataSummary::default();

    if let Ok(chunks) = parse_png_chunks(bytes) {
        for chunk in chunks {
            let type_str = String::from_utf8_lossy(&chunk.chunk_type).to_string();
            if !summary.found_chunk_types.contains(&type_str) {
                summary.found_chunk_types.push(type_str.clone());
            }

            match &chunk.chunk_type {
                b"IHDR" => {
                    if chunk.data.len() >= 8 {
                        summary.width = u32::from_be_bytes(chunk.data[0..4].try_into().unwrap());
                        summary.height = u32::from_be_bytes(chunk.data[4..8].try_into().unwrap());
                    }
                }
                b"tEXt" => {
                    if let Some(null_idx) = chunk.data.iter().position(|&b| b == 0) {
                        let key = String::from_utf8_lossy(&chunk.data[0..null_idx]).to_string();
                        let val = String::from_utf8_lossy(&chunk.data[null_idx + 1..]).to_string();
                        if key.eq_ignore_ascii_case("XML:com.adobe.xmp") {
                            summary.has_xmp = true;
                        }
                        summary.raw_chunks.push(RawChunkInfo {
                            key,
                            value: val,
                            chunk_type: "tEXt".to_string(),
                        });
                    }
                }
                b"zTXt" => {
                    if let Some(null_idx) = chunk.data.iter().position(|&b| b == 0) {
                        let key = String::from_utf8_lossy(&chunk.data[0..null_idx]).to_string();
                        // null_idx + 1 is compression method (usually 0)
                        if chunk.data.len() > null_idx + 2 {
                            let compressed_data = &chunk.data[null_idx + 2..];
                            let mut decoder = ZlibDecoder::new(compressed_data);
                            let mut decompressed = Vec::new();
                            if decoder.read_to_end(&mut decompressed).is_ok() {
                                let val = String::from_utf8_lossy(&decompressed).to_string();
                                if key.eq_ignore_ascii_case("XML:com.adobe.xmp") {
                                    summary.has_xmp = true;
                                }
                                summary.raw_chunks.push(RawChunkInfo {
                                    key,
                                    value: val,
                                    chunk_type: "zTXt".to_string(),
                                });
                            }
                        }
                    }
                }
                b"iTXt" => {
                    // Structure: keyword\0 comp_flag(1 byte) comp_method(1 byte) lang_tag\0 trans_key\0 text/compressed
                    if let Some(null1) = chunk.data.iter().position(|&b| b == 0) {
                        let key = String::from_utf8_lossy(&chunk.data[0..null1]).to_string();
                        if key.eq_ignore_ascii_case("XML:com.adobe.xmp") {
                            summary.has_xmp = true;
                        }
                        if chunk.data.len() > null1 + 2 {
                            let comp_flag = chunk.data[null1 + 1];
                            let rem = &chunk.data[null1 + 3..];
                            let mut scan_offset = 0;
                            // null for lang_tag
                            if let Some(l_null) = rem[scan_offset..].iter().position(|&b| b == 0) {
                                scan_offset += l_null + 1;
                                // null for trans_key
                                if let Some(t_null) =
                                    rem[scan_offset..].iter().position(|&b| b == 0)
                                {
                                    scan_offset += t_null + 1;
                                    let payload = &rem[scan_offset..];
                                    if comp_flag == 1 {
                                        let mut decoder = ZlibDecoder::new(payload);
                                        let mut decompressed = Vec::new();
                                        if decoder.read_to_end(&mut decompressed).is_ok() {
                                            let val =
                                                String::from_utf8_lossy(&decompressed).to_string();
                                            summary.raw_chunks.push(RawChunkInfo {
                                                key,
                                                value: val,
                                                chunk_type: "iTXt (compressed)".to_string(),
                                            });
                                        }
                                    } else {
                                        let val = String::from_utf8_lossy(payload).to_string();
                                        summary.raw_chunks.push(RawChunkInfo {
                                            key,
                                            value: val,
                                            chunk_type: "iTXt".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                b"eXIf" => {
                    summary.raw_exif = Some(chunk.data.to_vec());
                }
                b"iCCP" => {
                    summary.has_icc = true;
                }
                b"jumb" | b"caPI" | b"c2pa" | b"c2cs" | b"c2bi" | b"c2as" | b"dSIG" | b"JUMB" => {
                    summary.has_c2pa = true;
                }
                _ => {}
            }
        }
    }

    summary
}

/// Strips selected or all metadata chunks from a PNG file losslessly without re-encoding pixels.
///
/// Preserves critical visual chunks (`IHDR`, `PLTE`, `IDAT`, `IEND`), transparency (`tRNS`),
/// color gammas (`gAMA`, `sRGB`, `cHRM`), pixel density (`pHYs`), and APNG animation frames
/// (`acTL`, `fcTL`, `fdAT`).
///
/// # Errors
/// Returns an error if the input bytes do not constitute a valid PNG file.
pub fn clean_png_lossless(bytes: &[u8], options: &CleanOptions) -> Result<CleanResult, String> {
    let chunks = parse_png_chunks(bytes)?;
    let original_size = bytes.len();

    let mut output = Vec::with_capacity(original_size);
    output.extend_from_slice(PNG_MAGIC);

    for chunk in chunks {
        let chunk_type = &chunk.chunk_type;
        let mut keep = false;

        // Whitelist of essential rendering, color profile, and animation chunks
        match chunk_type {
            b"IHDR" | b"PLTE" | b"IDAT" | b"IEND" | b"tRNS" | b"cHRM" | b"gAMA" | b"sBIT"
            | b"sRGB" | b"pHYs" | b"acTL" | b"fcTL" | b"fdAT" => {
                keep = true;
            }
            b"iCCP" => {
                if !options.strip_all && !options.strip_icc_profile {
                    keep = true;
                }
            }
            _ => {
                if !options.strip_all {
                    match chunk_type {
                        b"tEXt" | b"zTXt" | b"iTXt" => {
                            // Extract key to check for XMP
                            let key =
                                if let Some(null_idx) = chunk.data.iter().position(|&b| b == 0) {
                                    String::from_utf8_lossy(&chunk.data[0..null_idx]).to_string()
                                } else {
                                    String::new()
                                };

                            let is_xmp = key.eq_ignore_ascii_case("XML:com.adobe.xmp");

                            if is_xmp && options.strip_xmp {
                                keep = false;
                            } else if !options.strip_ai_metadata && !options.strip_comments {
                                keep = true;
                            }
                        }
                        b"eXIf" => {
                            if !options.strip_exif && !options.strip_gps {
                                keep = true;
                            }
                        }
                        b"jumb" | b"caPI" | b"c2pa" | b"c2cs" | b"c2bi" | b"c2as" | b"dSIG"
                        | b"JUMB" => {
                            if !options.strip_c2pa {
                                keep = true;
                            }
                        }
                        _ => {
                            // Non-whitelisted unknown chunks are stripped
                            keep = false;
                        }
                    }
                }
            }
        }

        if keep {
            output.extend_from_slice(chunk.raw_slice);
        }
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

    // Construct a minimal valid 1x1 PNG byte array for testing
    fn create_dummy_png(include_text: bool) -> Vec<u8> {
        let mut png = Vec::new();
        png.extend_from_slice(PNG_MAGIC);

        // IHDR chunk: 1x1 8-bit RGBA
        let ihdr_data = [
            0x00, 0x00, 0x00, 0x01, // width: 1
            0x00, 0x00, 0x00, 0x01, // height: 1
            0x08, 0x06, 0x00, 0x00, 0x00, // 8-bit RGBA, deflate, no filter, non-interlaced
        ];
        append_chunk(&mut png, b"IHDR", &ihdr_data);

        if include_text {
            // tEXt chunk: prompt\0A beautiful cat
            let mut text_data = Vec::new();
            text_data.extend_from_slice(b"prompt\0A beautiful cat");
            append_chunk(&mut png, b"tEXt", &text_data);
        }

        // IDAT chunk: empty raw compressed scanline
        let idat_data = [0x78, 0x9c, 0x63, 0x60, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01];
        append_chunk(&mut png, b"IDAT", &idat_data);

        // IEND chunk
        append_chunk(&mut png, b"IEND", &[]);

        png
    }

    fn append_chunk(buffer: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
        let length = data.len() as u32;
        buffer.extend_from_slice(&length.to_be_bytes());
        buffer.extend_from_slice(chunk_type);
        buffer.extend_from_slice(data);

        let mut crc_hasher = crc32fast::Hasher::new();
        crc_hasher.update(chunk_type);
        crc_hasher.update(data);
        let crc = crc_hasher.finalize();
        buffer.extend_from_slice(&crc.to_be_bytes());
    }

    #[test]
    fn test_parse_png_chunks() {
        let png = create_dummy_png(true);
        let chunks = parse_png_chunks(&png).expect("Valid PNG should parse");
        assert_eq!(chunks.len(), 4); // IHDR, tEXt, IDAT, IEND
        assert_eq!(&chunks[0].chunk_type, b"IHDR");
        assert_eq!(&chunks[1].chunk_type, b"tEXt");
        assert_eq!(&chunks[2].chunk_type, b"IDAT");
        assert_eq!(&chunks[3].chunk_type, b"IEND");
    }

    #[test]
    fn test_extract_png_metadata() {
        let png = create_dummy_png(true);
        let summary = extract_png_metadata(&png);
        assert_eq!(summary.width, 1);
        assert_eq!(summary.height, 1);
        assert_eq!(summary.raw_chunks.len(), 1);
        assert_eq!(summary.raw_chunks[0].key, "prompt");
        assert_eq!(summary.raw_chunks[0].value, "A beautiful cat");
    }

    #[test]
    fn test_clean_png_lossless() {
        let png = create_dummy_png(true);
        let options = CleanOptions {
            strip_all: true,
            ..Default::default()
        };

        let result = clean_png_lossless(&png, &options).expect("Clean should succeed");
        assert!(result.success);
        assert!(result.cleaned_size < result.original_size);
        assert!(result.bytes_removed > 0);

        let cleaned_summary = extract_png_metadata(&result.cleaned_bytes);
        assert_eq!(cleaned_summary.raw_chunks.len(), 0);
        assert_eq!(cleaned_summary.width, 1);
        assert_eq!(cleaned_summary.height, 1);
    }
}
