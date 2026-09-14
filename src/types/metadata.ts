/**
 * Severity level of privacy or metadata exposure risk.
 */
export type RiskSeverity = 'high' | 'medium' | 'low' | 'info'

/**
 * Overall risk classification level for an image.
 */
export type RiskLevel = 'safe' | 'medium' | 'high'

/**
 * An individual privacy risk item identified during analysis.
 */
export interface RiskItem {
  category: string
  severity: RiskSeverity
  title: string
  description: string
}

/**
 * Privacy and information exposure risk assessment report.
 */
export interface RiskReport {
  score: number // 0 to 100 (100 = safe, 0 = critical leak)
  level: RiskLevel
  items: RiskItem[]
}

/**
 * Geolocation details extracted from EXIF GPS tags.
 */
export interface GpsInfo {
  latitude: number
  longitude: number
  altitude?: number
  formatted_coords: string
  osm_url: string
  google_maps_url: string
}

/**
 * Raw EXIF tag entry.
 */
export interface ExifTagItem {
  tag_name: string
  ifd: string
  value: string
}

/**
 * Camera shooting parameters and device information report.
 */
export interface ExifReport {
  camera_make?: string
  camera_model?: string
  lens_model?: string
  software?: string
  date_time?: string
  iso?: string
  f_number?: string
  exposure_time?: string
  focal_length?: string
  serial_number?: string
  user_comment?: string
  gps?: GpsInfo
  all_tags: ExifTagItem[]
  raw_size_bytes: number
}

/**
 * Content Credentials / C2PA provenance manifest report.
 */
export interface C2paReport {
  has_c2pa: boolean
  generator?: string
  claim_generator?: string
  signature_issuer?: string
  raw_manifest_summary?: string
  box_count: number
}

/**
 * Container text or metadata chunk entry.
 */
export interface RawChunkInfo {
  key: string
  value: string
  chunk_type: string
}

/**
 * AI generation parameters, prompts, and node workflows.
 */
export interface AiMetadata {
  platform?: string
  prompt?: string
  negative_prompt?: string
  steps?: number
  sampler?: string
  cfg_scale?: number
  seed?: number
  size?: string
  model_name?: string
  model_hash?: string
  denoising_strength?: number
  clip_skip?: number
  lora_tags: string[]
  extra_params: Record<string, string>
  comfy_workflow_json?: string
  comfy_prompt_json?: string
  raw_text_chunks: RawChunkInfo[]
}

/**
 * Comprehensive report containing all inspected metadata.
 */
export interface ImageMetadataReport {
  format: string
  width: number
  height: number
  file_size_bytes: number
  has_metadata: boolean
  exif?: ExifReport
  ai?: AiMetadata
  c2pa?: C2paReport
  icc_profile_present: boolean
  xmp_present: boolean
  risk_report: RiskReport
  raw_chunks_found: string[]
}

/**
 * Configuration options for lossless metadata stripping.
 */
export interface CleanOptions {
  strip_all: boolean
  strip_exif: boolean
  strip_gps: boolean
  strip_ai_metadata: boolean
  strip_c2pa: boolean
  strip_xmp: boolean
  strip_icc_profile: boolean
  strip_comments: boolean
}

/**
 * Default clean options (preserves ICC color profile by default).
 */
export const DEFAULT_CLEAN_OPTIONS: CleanOptions = {
  strip_all: true,
  strip_exif: true,
  strip_gps: true,
  strip_ai_metadata: true,
  strip_c2pa: true,
  strip_xmp: true,
  strip_icc_profile: false,
  strip_comments: true,
}

/**
 * Result returned after metadata sanitization.
 */
export interface CleanResult {
  success: boolean
  original_size: number
  cleaned_size: number
  bytes_removed: number
  percentage_reduced: number
  cleaned_bytes: Uint8Array
  error?: string
}
