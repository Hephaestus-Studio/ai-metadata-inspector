//! Core engine for image metadata inspection, AI prompt parsing, and lossless metadata sanitization.
//!
//! Exposes both native Rust APIs ([`inspect_image`], [`clean_image`]) and WebAssembly
//! bindings ([`inspect_image_wasm`], [`clean_image_wasm`]) for seamless JavaScript/TypeScript interop.

pub mod ai_parser;
pub mod c2pa_parser;
pub mod jpeg;
pub mod png;
pub mod types;
pub mod webp;

use ai_parser::parse_ai_metadata;
use c2pa_parser::detect_c2pa;
use exif::{Reader, Tag, Value};
use jpeg::{clean_jpeg_lossless, extract_jpeg_metadata};
use png::{clean_png_lossless, extract_png_metadata};
use types::{
    AiMetadata, CleanOptions, CleanResult, ExifReport, ExifTagItem, GpsInfo, ImageMetadataReport,
    RiskItem, RiskLevel, RiskReport, RiskSeverity,
};
use wasm_bindgen::prelude::*;
use webp::{clean_webp_lossless, extract_webp_metadata};

/// WebAssembly entry point: inspects binary image data and returns a serialized [`ImageMetadataReport`].
///
/// # Errors
/// Returns a JavaScript error if serialization to `JsValue` fails.
#[wasm_bindgen]
pub fn inspect_image_wasm(image_bytes: &[u8]) -> Result<JsValue, JsValue> {
    let report = inspect_image(image_bytes);
    serde_wasm_bindgen::to_value(&report).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// WebAssembly entry point: sanitizes and strips metadata according to [`CleanOptions`].
///
/// Returns a serialized [`CleanResult`] containing sanitized binary image bytes (`Uint8Array`).
///
/// # Errors
/// Returns a JavaScript error if deserialization of options or serialization to `JsValue` fails.
#[wasm_bindgen]
pub fn clean_image_wasm(image_bytes: &[u8], options_val: JsValue) -> Result<JsValue, JsValue> {
    let options: CleanOptions = if options_val.is_undefined() || options_val.is_null() {
        CleanOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options_val).unwrap_or_default()
    };

    let result = clean_image(image_bytes, &options);
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Inspects an image byte stream and generates a comprehensive [`ImageMetadataReport`].
///
/// Analyzes PNG, JPEG, and WebP containers for dimensions, EXIF camera parameters,
/// AI generation prompts (Stable Diffusion, ComfyUI, NovelAI, SwarmUI, InvokeAI),
/// Content Credentials (C2PA/JUMBF), ICC color profiles, and privacy risk scores.
pub fn inspect_image(bytes: &[u8]) -> ImageMetadataReport {
    let file_size_bytes = bytes.len();
    let format = detect_format(bytes);

    let mut width = 0;
    let mut height = 0;
    let mut raw_chunks = Vec::new();
    let mut raw_exif_bytes = None;
    let mut icc_present = false;
    let mut xmp_present = false;
    let mut c2pa_found = false;
    let mut raw_chunks_found = Vec::new();

    match format.as_str() {
        "PNG" => {
            let summary = extract_png_metadata(bytes);
            width = summary.width;
            height = summary.height;
            raw_chunks = summary.raw_chunks;
            raw_exif_bytes = summary.raw_exif;
            icc_present = summary.has_icc;
            xmp_present = summary.has_xmp;
            c2pa_found = summary.has_c2pa;
            raw_chunks_found = summary.found_chunk_types;
        }
        "JPEG" => {
            let summary = extract_jpeg_metadata(bytes);
            width = summary.width;
            height = summary.height;
            raw_chunks = summary.raw_chunks;
            raw_exif_bytes = summary.raw_exif;
            icc_present = summary.has_icc;
            xmp_present = summary.has_xmp;
            c2pa_found = summary.has_c2pa;
            raw_chunks_found = summary.found_markers;
        }
        "WebP" => {
            let summary = extract_webp_metadata(bytes);
            width = summary.width;
            height = summary.height;
            raw_chunks = summary.raw_chunks;
            raw_exif_bytes = summary.raw_exif;
            icc_present = summary.has_icc;
            xmp_present = summary.has_xmp;
            c2pa_found = summary.has_c2pa;
            raw_chunks_found = summary.found_chunk_types;
        }
        _ => {}
    }

    // Parse EXIF report if raw EXIF segment bytes were found
    let exif_report = raw_exif_bytes.as_deref().and_then(parse_exif_data);

    // Detect C2PA Content Credentials provenance
    let c2pa_report = if c2pa_found || format == "PNG" || format == "JPEG" || format == "WebP" {
        detect_c2pa(bytes)
    } else {
        None
    };

    // Parse AI Metadata from container text chunks and EXIF user comments
    let user_comment_str = exif_report.as_ref().and_then(|e| e.user_comment.as_deref());
    let ai_report = parse_ai_metadata(&raw_chunks, user_comment_str);

    // Compute Privacy and Metadata Risk Report
    let risk_report = compute_risk_report(
        &exif_report,
        &ai_report,
        &c2pa_report,
        icc_present,
        xmp_present,
    );

    let has_metadata = exif_report.is_some()
        || ai_report.is_some()
        || c2pa_report.is_some()
        || icc_present
        || xmp_present
        || !raw_chunks.is_empty();

    ImageMetadataReport {
        format,
        width,
        height,
        file_size_bytes,
        has_metadata,
        exif: exif_report,
        ai: ai_report,
        c2pa: c2pa_report,
        icc_profile_present: icc_present,
        xmp_present,
        risk_report,
        raw_chunks_found,
    }
}

/// Strips selected metadata fields from an image losslessly without re-encoding pixel bitstreams.
pub fn clean_image(bytes: &[u8], options: &CleanOptions) -> CleanResult {
    let format = detect_format(bytes);
    match format.as_str() {
        "PNG" => {
            clean_png_lossless(bytes, options).unwrap_or_else(|e| error_result(bytes.len(), e))
        }
        "JPEG" => {
            clean_jpeg_lossless(bytes, options).unwrap_or_else(|e| error_result(bytes.len(), e))
        }
        "WebP" => {
            clean_webp_lossless(bytes, options).unwrap_or_else(|e| error_result(bytes.len(), e))
        }
        _ => error_result(bytes.len(), format!("Unsupported image format: {}", format)),
    }
}

fn error_result(original_size: usize, error: String) -> CleanResult {
    CleanResult {
        success: false,
        original_size,
        cleaned_size: original_size,
        bytes_removed: 0,
        percentage_reduced: 0.0,
        cleaned_bytes: Vec::new(),
        error: Some(error),
    }
}

fn detect_format(bytes: &[u8]) -> String {
    if bytes.len() >= 8 && &bytes[0..8] == b"\x89PNG\r\n\x1a\n" {
        "PNG".to_string()
    } else if bytes.len() >= 2 && &bytes[0..2] == &[0xFF, 0xD8] {
        "JPEG".to_string()
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "WebP".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn parse_exif_data(raw_exif: &[u8]) -> Option<ExifReport> {
    let exif_data = Reader::new().read_raw(raw_exif.to_vec()).ok()?;

    let mut camera_make = None;
    let mut camera_model = None;
    let mut lens_model = None;
    let mut software = None;
    let mut date_time = None;
    let mut iso = None;
    let mut f_number = None;
    let mut exposure_time = None;
    let mut focal_length = None;
    let mut serial_number = None;
    let mut user_comment = None;

    let mut lat_deg: Option<f64> = None;
    let mut lat_ref = "N".to_string();
    let mut lon_deg: Option<f64> = None;
    let mut lon_ref = "E".to_string();
    let mut altitude: Option<f64> = None;

    let mut all_tags = Vec::new();

    for f in exif_data.fields() {
        let tag_name = format!("{}", f.tag);
        let ifd_str = format!("{:?}", f.ifd_num);
        let val_display = f.display_value().to_string();

        all_tags.push(ExifTagItem {
            tag_name: tag_name.clone(),
            ifd: ifd_str,
            value: val_display.clone(),
        });

        match f.tag {
            Tag::Make => camera_make = Some(val_display),
            Tag::Model => camera_model = Some(val_display),
            Tag::LensModel => lens_model = Some(val_display),
            Tag::Software => software = Some(val_display),
            Tag::DateTime | Tag::DateTimeOriginal | Tag::DateTimeDigitized => {
                if date_time.is_none() {
                    date_time = Some(val_display);
                }
            }
            Tag::PhotographicSensitivity => iso = Some(val_display),
            Tag::FNumber => f_number = Some(val_display),
            Tag::ExposureTime => exposure_time = Some(val_display),
            Tag::FocalLength => focal_length = Some(val_display),
            Tag::BodySerialNumber => serial_number = Some(val_display),
            Tag::UserComment => user_comment = Some(val_display),
            Tag::GPSLatitude => {
                if let Value::Rational(ref rats) = f.value {
                    if rats.len() == 3 {
                        let d = rats[0].num as f64 / rats[0].denom as f64;
                        let m = rats[1].num as f64 / rats[1].denom as f64;
                        let s = rats[2].num as f64 / rats[2].denom as f64;
                        lat_deg = Some(d + (m / 60.0) + (s / 3600.0));
                    }
                }
            }
            Tag::GPSLatitudeRef => {
                lat_ref = val_display.trim().to_uppercase();
            }
            Tag::GPSLongitude => {
                if let Value::Rational(ref rats) = f.value {
                    if rats.len() == 3 {
                        let d = rats[0].num as f64 / rats[0].denom as f64;
                        let m = rats[1].num as f64 / rats[1].denom as f64;
                        let s = rats[2].num as f64 / rats[2].denom as f64;
                        lon_deg = Some(d + (m / 60.0) + (s / 3600.0));
                    }
                }
            }
            Tag::GPSLongitudeRef => {
                lon_ref = val_display.trim().to_uppercase();
            }
            Tag::GPSAltitude => {
                if let Value::Rational(ref rats) = f.value {
                    if let Some(r) = rats.first() {
                        if r.denom != 0 {
                            altitude = Some(r.num as f64 / r.denom as f64);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let mut gps_info = None;
    if let (Some(mut lat), Some(mut lon)) = (lat_deg, lon_deg) {
        if lat_ref.starts_with('S') {
            lat = -lat;
        }
        if lon_ref.starts_with('W') {
            lon = -lon;
        }

        let formatted = format!("{:.6}°, {:.6}°", lat, lon);
        let osm_url = format!(
            "https://www.openstreetmap.org/?mlat={:.6}&mlon={:.6}#map=16/{:.6}/{:.6}",
            lat, lon, lat, lon
        );
        let google_maps_url = format!("https://www.google.com/maps?q={:.6},{:.6}", lat, lon);

        gps_info = Some(GpsInfo {
            latitude: lat,
            longitude: lon,
            altitude,
            formatted_coords: formatted,
            osm_url,
            google_maps_url,
        });
    }

    Some(ExifReport {
        camera_make,
        camera_model,
        lens_model,
        software,
        date_time,
        iso,
        f_number,
        exposure_time,
        focal_length,
        serial_number,
        user_comment,
        gps: gps_info,
        all_tags,
        raw_size_bytes: raw_exif.len(),
    })
}

fn compute_risk_report(
    exif: &Option<ExifReport>,
    ai: &Option<AiMetadata>,
    c2pa: &Option<types::C2paReport>,
    _icc: bool,
    _xmp: bool,
) -> RiskReport {
    let mut score = 100i32;
    let mut items = Vec::new();

    if let Some(e) = exif {
        if let Some(gps) = &e.gps {
            score -= 40;
            items.push(RiskItem {
                category: "GPS Location".to_string(),
                severity: RiskSeverity::High,
                title: "Sensitive GPS Coordinates".to_string(),
                description: format!(
                    "Exact geolocation coordinates exposed ({})",
                    gps.formatted_coords
                ),
            });
        }

        if let Some(serial) = &e.serial_number {
            score -= 20;
            items.push(RiskItem {
                category: "Device Fingerprint".to_string(),
                severity: RiskSeverity::High,
                title: "Camera Serial Number".to_string(),
                description: format!("Hardware identifier exposed: {}", serial),
            });
        }

        if e.camera_make.is_some() || e.camera_model.is_some() {
            score -= 10;
            let device = format!(
                "{} {}",
                e.camera_make.clone().unwrap_or_default(),
                e.camera_model.clone().unwrap_or_default()
            )
            .trim()
            .to_string();
            items.push(RiskItem {
                category: "Camera Hardware".to_string(),
                severity: RiskSeverity::Low,
                title: "Camera Model Details".to_string(),
                description: format!("Device model revealed: {}", device),
            });
        }

        if let Some(dt) = &e.date_time {
            score -= 5;
            items.push(RiskItem {
                category: "Timestamp".to_string(),
                severity: RiskSeverity::Low,
                title: "Capture Timestamp".to_string(),
                description: format!("Date and time of capture revealed: {}", dt),
            });
        }
    }

    if let Some(a) = ai {
        if a.prompt.is_some() || a.comfy_workflow_json.is_some() {
            score -= 15;
            items.push(RiskItem {
                category: "AI Generation Metadata".to_string(),
                severity: RiskSeverity::Medium,
                title: "AI Generation Parameters & Prompts".to_string(),
                description: format!(
                    "Contains full prompts, seed, checkpoint model ({})",
                    a.platform
                        .clone()
                        .unwrap_or_else(|| "AI Engine".to_string())
                ),
            });
        }
    }

    if let Some(c) = c2pa {
        if c.has_c2pa {
            score -= 15;
            items.push(RiskItem {
                category: "Digital Provenance".to_string(),
                severity: RiskSeverity::Medium,
                title: "C2PA / Content Credentials Signature".to_string(),
                description:
                    "Contains digital publishing provenance history and JUMBF signing certificate."
                        .to_string(),
            });
        }
    }

    let final_score = score.clamp(0, 100) as u32;
    let level = if final_score >= 85 {
        RiskLevel::Safe
    } else if final_score >= 50 {
        RiskLevel::Medium
    } else {
        RiskLevel::High
    };

    RiskReport {
        score: final_score,
        level,
        items,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crc32fast::Hasher;

    fn create_test_png(text_key: &str, text_val: &str) -> Vec<u8> {
        let mut png = Vec::new();
        png.extend_from_slice(b"\x89PNG\r\n\x1a\n");

        let mut ihdr_data = Vec::new();
        ihdr_data.extend_from_slice(&1u32.to_be_bytes());
        ihdr_data.extend_from_slice(&1u32.to_be_bytes());
        ihdr_data.push(8);
        ihdr_data.push(6);
        ihdr_data.push(0);
        ihdr_data.push(0);
        ihdr_data.push(0);
        write_png_chunk(&mut png, b"IHDR", &ihdr_data);

        let mut text_data = Vec::new();
        text_data.extend_from_slice(text_key.as_bytes());
        text_data.push(0);
        text_data.extend_from_slice(text_val.as_bytes());
        write_png_chunk(&mut png, b"tEXt", &text_data);

        let idat_data = vec![0x78, 0x9c, 0x63, 0x60, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01];
        write_png_chunk(&mut png, b"IDAT", &idat_data);
        write_png_chunk(&mut png, b"IEND", &[]);

        png
    }

    fn write_png_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        out.extend_from_slice(chunk_type);
        out.extend_from_slice(data);

        let mut hasher = Hasher::new();
        hasher.update(chunk_type);
        hasher.update(data);
        let crc = hasher.finalize();
        out.extend_from_slice(&crc.to_be_bytes());
    }

    #[test]
    fn test_sd_metadata_parse_and_clean() {
        let sd_params = "masterpiece, best quality, cyber girl in neon city\nNegative prompt: bad hands, blurry\nSteps: 28, Sampler: DPM++ 2M Karras, CFG scale: 7.5, Seed: 987654321, Size: 832x1216, Model: dreamshaper_v8";
        let png_bytes = create_test_png("parameters", sd_params);

        let report = inspect_image(&png_bytes);
        assert_eq!(report.format, "PNG");
        assert_eq!(report.width, 1);
        assert_eq!(report.height, 1);
        assert!(report.has_metadata);

        let ai = report.ai.expect("AI metadata should be detected");
        assert_eq!(
            ai.platform.unwrap(),
            "Stable Diffusion (A1111 / WebUI / Forge)"
        );
        assert_eq!(
            ai.prompt.unwrap(),
            "masterpiece, best quality, cyber girl in neon city"
        );
        assert_eq!(ai.negative_prompt.unwrap(), "bad hands, blurry");
        assert_eq!(ai.steps.unwrap(), 28);
        assert_eq!(ai.sampler.unwrap(), "DPM++ 2M Karras");
        assert_eq!(ai.cfg_scale.unwrap(), 7.5);
        assert_eq!(ai.seed.unwrap(), 987654321);
        assert_eq!(ai.model_name.unwrap(), "dreamshaper_v8");

        let clean_opts = CleanOptions::default();
        let clean_res = clean_image(&png_bytes, &clean_opts);
        assert!(clean_res.success);
        assert!(clean_res.bytes_removed > 0);
        assert!(clean_res.cleaned_size < clean_res.original_size);

        let cleaned_report = inspect_image(&clean_res.cleaned_bytes);
        assert_eq!(cleaned_report.format, "PNG");
        assert!(!cleaned_report.has_metadata);
        assert_eq!(cleaned_report.risk_report.score, 100);
        assert_eq!(cleaned_report.risk_report.level, RiskLevel::Safe);
    }

    #[test]
    fn test_comfyui_json_parsing() {
        let prompt_json = r#"{
            "3": {
                "class_type": "KSampler",
                "inputs": {
                    "cfg": 8.0,
                    "denoise": 1.0,
                    "sampler_name": "euler_ancestral",
                    "scheduler": "normal",
                    "seed": 1234567,
                    "steps": 25
                }
            },
            "4": {
                "class_type": "CheckpointLoaderSimple",
                "inputs": {
                    "ckpt_name": "v1-5-pruned-emaonly.safetensors"
                }
            },
            "6": {
                "class_type": "CLIPTextEncode",
                "inputs": {
                    "text": "epic dragon flying over mystical mountains, 8k wallpaper"
                }
            },
            "7": {
                "class_type": "CLIPTextEncode",
                "inputs": {
                    "text": "low quality, watermark, blurry, deformed"
                }
            }
        }"#;

        let png_bytes = create_test_png("prompt", prompt_json);
        let report = inspect_image(&png_bytes);
        let ai = report.ai.expect("AI metadata should be found for ComfyUI");

        assert_eq!(ai.platform.unwrap(), "ComfyUI");
        assert_eq!(
            ai.prompt.unwrap(),
            "epic dragon flying over mystical mountains, 8k wallpaper"
        );
        assert_eq!(
            ai.negative_prompt.unwrap(),
            "low quality, watermark, blurry, deformed"
        );
        assert_eq!(ai.steps.unwrap(), 25);
        assert_eq!(ai.seed.unwrap(), 1234567);
    }

    #[test]
    fn test_c2pa_stripping() {
        let c2pa_manifest =
            r#"claim_generator_info{"name":"OpenAI Media Service API","icon":"curl"}"#;
        let png_bytes = create_test_png("c2pa", c2pa_manifest);

        let report = inspect_image(&png_bytes);
        assert!(report.c2pa.is_some() || report.has_metadata);

        let clean_opts = CleanOptions::default();
        let clean_res = clean_image(&png_bytes, &clean_opts);
        assert!(clean_res.success);

        let cleaned_report = inspect_image(&clean_res.cleaned_bytes);
        assert!(!cleaned_report.has_metadata);
        assert!(cleaned_report.c2pa.is_none());
        assert_eq!(cleaned_report.risk_report.score, 100);
        assert_eq!(cleaned_report.risk_report.level, RiskLevel::Safe);
    }
}
