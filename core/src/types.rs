//! Data structures and types for image metadata inspection, privacy risk evaluation,
//! and metadata sanitization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Severity level of a detected privacy or metadata risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskSeverity {
    /// Critical or sensitive information exposed (e.g. GPS coordinates, personal serial numbers).
    High,
    /// Moderate exposure (e.g. detailed prompts, camera software/model).
    Medium,
    /// Minor exposure (e.g. generation parameters, timestamps).
    Low,
    /// Informational note without immediate privacy danger.
    Info,
}

impl Default for RiskSeverity {
    fn default() -> Self {
        Self::Info
    }
}

/// Overall risk level categorization for an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Safe to share publicly with minimal or no sensitive metadata.
    Safe,
    /// Contains some identifiable information; review suggested.
    Medium,
    /// Contains sensitive information (e.g., GPS, camera serials); cleaning recommended.
    High,
}

impl Default for RiskLevel {
    fn default() -> Self {
        Self::Safe
    }
}

/// A specific risk item identified during image metadata analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskItem {
    /// Category of the risk (e.g. "Location", "Device", "AI Generation", "Author").
    pub category: String,
    /// Severity level of the risk.
    pub severity: RiskSeverity,
    /// Short human-readable title of the risk.
    pub title: String,
    /// Detailed explanation of why this metadata item may pose a risk.
    pub description: String,
}

/// Comprehensive privacy and security risk assessment report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskReport {
    /// Privacy safety score from 0 to 100 (100 = completely clean / safe, 0 = critical leak).
    pub score: u32,
    /// Categorized risk level based on the computed score and detected items.
    pub level: RiskLevel,
    /// List of specific identified risk items.
    pub items: Vec<RiskItem>,
}

impl Default for RiskReport {
    fn default() -> Self {
        Self {
            score: 100,
            level: RiskLevel::Safe,
            items: Vec::new(),
        }
    }
}

/// Geolocation details extracted from EXIF GPS tags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsInfo {
    /// Latitude in decimal degrees.
    pub latitude: f64,
    /// Longitude in decimal degrees.
    pub longitude: f64,
    /// Altitude in meters above sea level, if available.
    pub altitude: Option<f64>,
    /// Human-readable formatted coordinate string (e.g. "37°46'29.7\"N 122°25'09.8\"W").
    pub formatted_coords: String,
    /// Pre-generated link to OpenStreetMap for this location.
    pub osm_url: String,
    /// Pre-generated link to Google Maps for this location.
    pub google_maps_url: String,
}

/// An individual raw EXIF tag representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExifTagItem {
    /// Tag name or human-readable description (e.g. "Model", "FNumber").
    pub tag_name: String,
    /// Image File Directory (IFD) where the tag was found (e.g. "0th", "Exif", "GPS").
    pub ifd: String,
    /// Formatted display value of the tag.
    pub value: String,
}

/// Report containing extracted EXIF camera metadata and shooting parameters.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExifReport {
    /// Camera manufacturer / make (e.g. "Canon", "Sony", "Apple").
    pub camera_make: Option<String>,
    /// Camera model name (e.g. "EOS R5", "iPhone 15 Pro").
    pub camera_model: Option<String>,
    /// Lens model used to capture the image.
    pub lens_model: Option<String>,
    /// Processing or capture software (e.g. "Adobe Photoshop", "Capture One").
    pub software: Option<String>,
    /// Date and time when the photo was taken or digitized.
    pub date_time: Option<String>,
    /// ISO speed rating.
    pub iso: Option<String>,
    /// Lens aperture (F-number).
    pub f_number: Option<String>,
    /// Exposure time / shutter speed (e.g. "1/250s").
    pub exposure_time: Option<String>,
    /// Focal length of the lens.
    pub focal_length: Option<String>,
    /// Camera hardware serial number.
    pub serial_number: Option<String>,
    /// Custom user comments embedded in EXIF.
    pub user_comment: Option<String>,
    /// Extracted GPS location information, if available.
    pub gps: Option<GpsInfo>,
    /// Complete list of all parsed EXIF tags.
    pub all_tags: Vec<ExifTagItem>,
    /// Total raw byte size of the EXIF segment.
    pub raw_size_bytes: usize,
}

/// Report for Content Credentials / C2PA (Coalition for Content Provenance and Authenticity) manifests.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct C2paReport {
    /// Indicates whether a C2PA manifest / JUMBF box was detected.
    pub has_c2pa: bool,
    /// Software or hardware generator of the assertion (e.g. "Adobe Firefly", "Truepic").
    pub generator: Option<String>,
    /// Claim generator application name and version.
    pub claim_generator: Option<String>,
    /// Digital signature issuer / certificate authority.
    pub signature_issuer: Option<String>,
    /// Summary or serialized representation of the raw C2PA manifest.
    pub raw_manifest_summary: Option<String>,
    /// Total count of JUMBF metadata boxes found.
    pub box_count: usize,
}

/// Raw key-value metadata chunk extracted directly from image container formats (e.g. PNG text chunks).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawChunkInfo {
    /// Key or identifier for the chunk (e.g. "parameters", "prompt", "workflow").
    pub key: String,
    /// Decoded text or representation of chunk payload.
    pub value: String,
    /// Container chunk type identifier (e.g. "tEXt", "zTXt", "iTXt").
    pub chunk_type: String,
}

/// AI generation parameters and workflow metadata extracted from image files.
///
/// Supports major AI art platforms including Stable Diffusion (Automatic1111, Forge),
/// ComfyUI, NovelAI, Midjourney, DALL-E, Fooocus, and InvokeAI.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiMetadata {
    /// Detected AI generator or platform name (e.g. "Stable Diffusion (A1111)", "ComfyUI", "NovelAI").
    pub platform: Option<String>,
    /// Primary positive generation prompt text.
    pub prompt: Option<String>,
    /// Negative prompt text specifying elements to exclude.
    pub negative_prompt: Option<String>,
    /// Number of denoising / diffusion sampling steps.
    pub steps: Option<u32>,
    /// Sampling algorithm name (e.g. "Euler a", "DPM++ 2M Karras").
    pub sampler: Option<String>,
    /// Classifier-Free Guidance (CFG) scale value.
    pub cfg_scale: Option<f64>,
    /// Generation random seed.
    pub seed: Option<i64>,
    /// Output generation resolution (e.g. "1024x1024").
    pub size: Option<String>,
    /// Name or filename of the checkpoint / base model.
    pub model_name: Option<String>,
    /// Short hash of the model weights (e.g. "7f80514a67").
    pub model_hash: Option<String>,
    /// Denoising strength for image-to-image or highres fix pipelines.
    pub denoising_strength: Option<f64>,
    /// CLIP skip layer offset.
    pub clip_skip: Option<u32>,
    /// List of LoRA (Low-Rank Adaptation) models and their weights detected in prompt.
    pub lora_tags: Vec<String>,
    /// Additional generation parameters not mapped to standard fields.
    pub extra_params: HashMap<String, String>,
    /// Raw serialized ComfyUI node graph workflow JSON string.
    pub comfy_workflow_json: Option<String>,
    /// Raw serialized ComfyUI execution prompt JSON string.
    pub comfy_prompt_json: Option<String>,
    /// Collection of all raw text-based metadata chunks found in the image.
    pub raw_text_chunks: Vec<RawChunkInfo>,
}

/// Comprehensive report containing all inspected metadata and analysis for an image file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadataReport {
    /// Detected image container format (e.g. "PNG", "JPEG", "WebP").
    pub format: String,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Total file size of the input image in bytes.
    pub file_size_bytes: usize,
    /// Flag indicating whether any metadata chunks or headers were detected.
    pub has_metadata: bool,
    /// Extracted EXIF camera and shooting information, if present.
    pub exif: Option<ExifReport>,
    /// Extracted AI generation prompts and workflow parameters, if present.
    pub ai: Option<AiMetadata>,
    /// Extracted Content Credentials / C2PA provenance information, if present.
    pub c2pa: Option<C2paReport>,
    /// Indicates whether an embedded ICC color profile is present.
    pub icc_profile_present: bool,
    /// Indicates whether an XMP (Extensible Metadata Platform) data packet is present.
    pub xmp_present: bool,
    /// Privacy and information exposure risk evaluation.
    pub risk_report: RiskReport,
    /// List of all raw metadata chunk / marker types identified in the container.
    pub raw_chunks_found: Vec<String>,
}

/// Configuration options for cleaning and stripping metadata from image files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanOptions {
    /// If true, strips all non-essential metadata chunks regardless of individual flags.
    pub strip_all: bool,
    /// If true, removes EXIF metadata segments.
    pub strip_exif: bool,
    /// If true, specifically removes GPS location data while retaining other EXIF tags.
    pub strip_gps: bool,
    /// If true, strips AI generation prompts, workflow graphs, and parameters.
    pub strip_ai_metadata: bool,
    /// If true, strips Content Credentials and C2PA provenance data.
    pub strip_c2pa: bool,
    /// If true, removes XMP data packets.
    pub strip_xmp: bool,
    /// If true, removes embedded ICC color profiles. (Default: false to preserve color accuracy).
    pub strip_icc_profile: bool,
    /// If true, strips generic text comments and description chunks.
    pub strip_comments: bool,
}

impl Default for CleanOptions {
    fn default() -> Self {
        Self {
            strip_all: true,
            strip_exif: true,
            strip_gps: true,
            strip_ai_metadata: true,
            strip_c2pa: true,
            strip_xmp: true,
            strip_icc_profile: false,
            strip_comments: true,
        }
    }
}

/// Result returned after executing image metadata sanitization / stripping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanResult {
    /// Indicates whether the cleaning operation completed successfully.
    pub success: bool,
    /// Original image size in bytes before cleaning.
    pub original_size: usize,
    /// Sanitized image size in bytes after stripping metadata.
    pub cleaned_size: usize,
    /// Total bytes removed from the file.
    pub bytes_removed: usize,
    /// Percentage reduction in file size (0.0 to 100.0).
    pub percentage_reduced: f64,
    /// Sanitized image binary bytes, optimized for WebAssembly interop.
    #[serde(with = "serde_bytes")]
    pub cleaned_bytes: Vec<u8>,
    /// Error message string if cleaning failed.
    pub error: Option<String>,
}
