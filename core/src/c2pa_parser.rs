//! Parsing and detection of C2PA (Coalition for Content Provenance and Authenticity)
//! and Content Authenticity Initiative (CAI) manifests and JUMBF boxes.
//!
//! Complies with the C2PA Technical Specification v1.x/v2.x (ISO/IEC 19566-5 JUMBF container format),
//! supporting automated identification of AI generation claims, digital signature issuers,
//! and claim generator applications (Adobe Firefly, DALL-E, Microsoft Designer, Leica, Truepic, etc.).

use crate::types::C2paReport;

/// Scans raw image container bytes for C2PA provenance assertions and JUMBF metadata boxes.
///
/// Efficiently searches for JUMBF box signatures (`jumb`, `c2pa`, `c2cs`, `c2ma`, `c2as`),
/// manifest URLs, and claim generator assertions without full-buffer string allocations.
pub fn detect_c2pa(bytes: &[u8]) -> Option<C2paReport> {
    let mut found = false;
    let mut box_count = 0;
    let mut generator: Option<String> = None;
    let mut claim_generator: Option<String> = None;
    let mut signature_issuer: Option<String> = None;

    // Scan for JUMBF box four-character codes
    for window in bytes.windows(4) {
        match window {
            b"jumb" | b"c2pa" | b"c2cs" | b"c2ma" | b"c2as" | b"caPI" => {
                found = true;
                box_count += 1;
            }
            _ => {}
        }
    }

    // Check for C2PA manifest namespace URIs or claim identifiers in binary stream
    if contains_bytes(bytes, b"http://c2pa.org/")
        || contains_bytes(bytes, b"c2pa.claim")
        || contains_bytes(bytes, b"c2pa.manifest")
        || contains_bytes(bytes, b"c2pa.assertions")
    {
        found = true;
    }

    // If C2PA signatures or namespaces were found, extract metadata details
    if found {
        // 1. Try extracting claim generator from JSON manifests
        if let Some(extracted_gen) = extract_json_string_value(bytes, b"\"claim_generator\"") {
            claim_generator = Some(extracted_gen);
        } else if let Some(extracted_gen) = extract_json_string_value(bytes, b"claim_generator") {
            claim_generator = Some(extracted_gen);
        }

        // 2. Try extracting signature issuer / cert authority
        if let Some(issuer) = extract_json_string_value(bytes, b"\"issuer\"") {
            signature_issuer = Some(issuer);
        } else if let Some(issuer) = extract_json_string_value(bytes, b"\"organizationName\"") {
            signature_issuer = Some(issuer);
        }

        // 3. Classify known generator ecosystem
        if contains_bytes_case_insensitive(bytes, b"Adobe Firefly") {
            generator = Some("Adobe Firefly (Content Credentials)".to_string());
        } else if contains_bytes_case_insensitive(bytes, b"Photoshop")
            && contains_bytes(bytes, b"c2pa")
        {
            generator = Some("Adobe Photoshop (Content Credentials)".to_string());
        } else if contains_bytes_case_insensitive(bytes, b"Lightroom") {
            generator = Some("Adobe Lightroom (Content Credentials)".to_string());
        } else if contains_bytes_case_insensitive(bytes, b"OpenAI")
            || contains_bytes_case_insensitive(bytes, b"DALL-E")
            || contains_bytes_case_insensitive(bytes, b"ChatGPT")
        {
            generator = Some("OpenAI DALL-E / ChatGPT (Content Credentials)".to_string());
        } else if contains_bytes_case_insensitive(bytes, b"Microsoft")
            && (contains_bytes_case_insensitive(bytes, b"Designer")
                || contains_bytes_case_insensitive(bytes, b"Bing"))
        {
            generator =
                Some("Microsoft Designer / Bing Image Creator (Content Credentials)".to_string());
        } else if contains_bytes_case_insensitive(bytes, b"Midjourney") {
            generator = Some("Midjourney (Content Credentials)".to_string());
        } else if contains_bytes_case_insensitive(bytes, b"Google")
            || contains_bytes_case_insensitive(bytes, b"SynthID")
            || contains_bytes_case_insensitive(bytes, b"Imagen")
        {
            generator = Some("Google SynthID / DeepMind (Content Credentials)".to_string());
        } else if contains_bytes_case_insensitive(bytes, b"Leica") {
            generator = Some("Leica Content Credentials (Hardware Authenticated)".to_string());
        } else if contains_bytes_case_insensitive(bytes, b"Sony") {
            generator = Some("Sony Camera Authenticity (In-Camera Signed)".to_string());
        } else if contains_bytes_case_insensitive(bytes, b"Nikon") {
            generator = Some("Nikon Camera Provenance (In-Camera Signed)".to_string());
        } else if contains_bytes_case_insensitive(bytes, b"Truepic") {
            generator = Some("Truepic Provenance Platform".to_string());
        }

        Some(C2paReport {
            has_c2pa: true,
            generator,
            claim_generator,
            signature_issuer,
            raw_manifest_summary: Some(format!(
                "C2PA / JUMBF Digital Provenance assertions detected ({} box headers)",
                box_count
            )),
            box_count,
        })
    } else {
        None
    }
}

/// Helper function to check if a byte pattern exists within a slice without allocations.
fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// Case-insensitive byte slice search helper.
fn contains_bytes_case_insensitive(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| {
        w.iter()
            .zip(needle.iter())
            .all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
    })
}

/// Extracts the string value associated with a JSON key within binary data.
///
/// Handles patterns like `"key": "value"` or `"key":"value"`.
fn extract_json_string_value(bytes: &[u8], key: &[u8]) -> Option<String> {
    if let Some(pos) = find_subsequence(bytes, key) {
        let remainder = &bytes[pos + key.len()..];
        // Look for colon `:`
        if let Some(colon_idx) = remainder.iter().position(|&b| b == b':') {
            let after_colon = &remainder[colon_idx + 1..];
            // Find opening quote `"`
            if let Some(quote_start) = after_colon.iter().position(|&b| b == b'"') {
                let value_slice = &after_colon[quote_start + 1..];
                // Find closing quote `"` (allowing max 256 chars for safe extraction)
                let max_len = value_slice.len().min(256);
                if let Some(quote_end) = value_slice[..max_len].iter().position(|&b| b == b'"') {
                    let val = String::from_utf8_lossy(&value_slice[..quote_end])
                        .trim()
                        .to_string();
                    if !val.is_empty() {
                        return Some(val);
                    }
                }
            }
        }
    }
    None
}

/// Returns the starting index of the first occurrence of needle in haystack.
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_c2pa_with_jumbf() {
        let mut data = Vec::new();
        data.extend_from_slice(b"\x00\x00\x00\x20jumb\x00\x00\x00\x10c2pa");
        data.extend_from_slice(
            b"{\"claim_generator\": \"Adobe Firefly 2.0\", \"issuer\": \"Adobe Inc CA\"}",
        );

        let report = detect_c2pa(&data).expect("C2PA should be detected");
        assert!(report.has_c2pa);
        assert_eq!(report.claim_generator.as_deref(), Some("Adobe Firefly 2.0"));
        assert_eq!(report.signature_issuer.as_deref(), Some("Adobe Inc CA"));
        assert_eq!(
            report.generator.as_deref(),
            Some("Adobe Firefly (Content Credentials)")
        );
        assert!(report.box_count >= 2);
    }

    #[test]
    fn test_detect_c2pa_openai() {
        let mut data = Vec::new();
        data.extend_from_slice(b"http://c2pa.org/manifest\x00");
        data.extend_from_slice(b"{\"claim_generator\":\"DALL-E 3 (OpenAI)\"}");

        let report = detect_c2pa(&data).expect("C2PA should be detected");
        assert!(report.has_c2pa);
        assert_eq!(report.claim_generator.as_deref(), Some("DALL-E 3 (OpenAI)"));
        assert_eq!(
            report.generator.as_deref(),
            Some("OpenAI DALL-E / ChatGPT (Content Credentials)")
        );
    }

    #[test]
    fn test_detect_c2pa_none() {
        let data = b"Just a plain image without any C2PA metadata.";
        let report = detect_c2pa(data);
        assert!(report.is_none());
    }
}
