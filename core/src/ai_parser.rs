//! Parsing and extraction of generative AI prompts, models, and workflows.
//!
//! Supports all major AI image generation tools and ecosystems:
//! - **Stable Diffusion WebUI (Automatic1111, Forge, SD.Next, reForge)**: Parameter text block with positive/negative prompts, sampler, steps, CFG, seed, size, model, hash, LoRA tags, and hires fix.
//! - **ComfyUI**: Node execution graph (`prompt`) and visual editor workflow (`workflow`), resolving conditioning links between `KSampler` and `CLIPTextEncode` nodes.
//! - **NovelAI**: Metadata JSON containing prompts, undesired content (`uc`), seed, scale, and sampler.
//! - **InvokeAI**: `invokeai_metadata` and `sd-metadata` parameter bundles.
//! - **SwarmUI**: `sui_image_params` JSON payload.
//! - **Fooocus / Midjourney / OpenAI DALL-E / ChatGPT**: Text prompts and generation descriptors.

use crate::types::{AiMetadata, RawChunkInfo};
use std::collections::HashMap;

/// Analyzes container text chunks and EXIF user comments to identify and extract AI metadata.
///
/// Returns an [`AiMetadata`] struct populated with prompts, hyperparameters, model names,
/// LoRA configurations, and workflow graphs if AI generation signatures are discovered.
pub fn parse_ai_metadata(
    raw_chunks: &[RawChunkInfo],
    exif_user_comment: Option<&str>,
) -> Option<AiMetadata> {
    let mut ai = AiMetadata::default();
    ai.raw_text_chunks = raw_chunks.to_vec();

    let mut detected = false;

    // 1. Check for ComfyUI chunks ("prompt", "workflow")
    let mut comfy_prompt_str = None;
    let mut comfy_workflow_str = None;

    for chunk in raw_chunks {
        let key_lower = chunk.key.to_lowercase();
        if key_lower == "prompt" {
            comfy_prompt_str = Some(chunk.value.clone());
        } else if key_lower == "workflow" {
            comfy_workflow_str = Some(chunk.value.clone());
        }
    }

    if comfy_prompt_str.is_some() || comfy_workflow_str.is_some() {
        if let Some(prompt_json) = comfy_prompt_str.as_deref() {
            if parse_comfyui_json(prompt_json, &mut ai) {
                detected = true;
                ai.platform = Some("ComfyUI".to_string());
                ai.comfy_prompt_json = Some(prompt_json.to_string());
            }
        }
        if let Some(workflow_json) = comfy_workflow_str {
            ai.comfy_workflow_json = Some(workflow_json);
            if ai.platform.is_none() {
                ai.platform = Some("ComfyUI".to_string());
                detected = true;
            }
        }
    }

    // 2. Check for SwarmUI ("sui_image_params")
    if !detected {
        for chunk in raw_chunks {
            let key_lower = chunk.key.to_lowercase();
            if key_lower == "sui_image_params" {
                if parse_swarmui_json(&chunk.value, &mut ai) {
                    ai.platform = Some("SwarmUI".to_string());
                    detected = true;
                    break;
                }
            }
        }
    }

    // 3. Check for InvokeAI ("invokeai_metadata" or "sd-metadata")
    if !detected {
        for chunk in raw_chunks {
            let key_lower = chunk.key.to_lowercase();
            if key_lower == "invokeai_metadata" {
                if parse_invokeai_json(&chunk.value, &mut ai) {
                    ai.platform = Some("InvokeAI".to_string());
                    detected = true;
                    break;
                }
            }
        }
    }

    // 4. Check for A1111 / Forge "parameters" chunk or Exif UserComment
    let mut sd_parameters_str = None;
    for chunk in raw_chunks {
        let key_lower = chunk.key.to_lowercase();
        if key_lower == "parameters"
            || key_lower == "sd-metadata"
            || key_lower == "generation_data"
            || key_lower == "comment" && chunk.value.contains("Steps:")
        {
            sd_parameters_str = Some(chunk.value.clone());
            break;
        }
    }

    if sd_parameters_str.is_none() {
        if let Some(comment) = exif_user_comment {
            if comment.contains("Steps:")
                || comment.contains("Negative prompt:")
                || comment.contains("Sampler:")
            {
                sd_parameters_str = Some(comment.to_string());
            }
        }
    }

    if let Some(params_text) = sd_parameters_str {
        if parse_a1111_parameters(&params_text, &mut ai) {
            detected = true;
            if ai.platform.is_none() {
                ai.platform = Some("Stable Diffusion (A1111 / WebUI / Forge)".to_string());
            }
        }
    }

    // 5. Check for NovelAI ("Comment" JSON or "Description")
    if !detected {
        for chunk in raw_chunks {
            let key_lower = chunk.key.to_lowercase();
            if key_lower == "comment" || key_lower == "description" {
                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&chunk.value) {
                    if let Some(prompt) = json_val.get("prompt").and_then(|v| v.as_str()) {
                        ai.prompt = Some(prompt.to_string());
                        ai.negative_prompt = json_val
                            .get("uc")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        ai.steps = json_val
                            .get("steps")
                            .and_then(|v| v.as_u64())
                            .map(|u| u as u32);
                        ai.cfg_scale = json_val.get("scale").and_then(|v| v.as_f64());
                        ai.seed = json_val.get("seed").and_then(|v| v.as_i64());
                        ai.sampler = json_val
                            .get("sampler")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        ai.platform = Some("NovelAI".to_string());
                        detected = true;
                    }
                }
            }
        }
    }

    // 6. Check for Fooocus, Midjourney, DALL-E signatures
    if !detected {
        for chunk in raw_chunks {
            let val = &chunk.value;
            if val.contains("Fooocus") || chunk.key.eq_ignore_ascii_case("fooocus") {
                ai.prompt = Some(val.clone());
                ai.platform = Some("Fooocus".to_string());
                detected = true;
                break;
            } else if val.contains("Midjourney") || val.contains("--v ") || val.contains("--ar ") {
                ai.prompt = Some(val.clone());
                ai.platform = Some("Midjourney".to_string());
                detected = true;
                break;
            } else if val.contains("DALL-E") || val.contains("ChatGPT") {
                ai.prompt = Some(val.clone());
                ai.platform = Some("OpenAI DALL-E".to_string());
                detected = true;
                break;
            }
        }
    }

    if detected {
        Some(ai)
    } else if !raw_chunks.is_empty() {
        // Retain parsed raw text chunks even if no known platform pattern was identified
        Some(ai)
    } else {
        None
    }
}

/// Parses standard Automatic1111 / WebUI / Forge parameter text blocks.
pub fn parse_a1111_parameters(text: &str, ai: &mut AiMetadata) -> bool {
    let mut lines = text.lines().peekable();
    let mut prompt_lines = Vec::new();
    let mut neg_prompt_lines = Vec::new();
    let mut params_line = None;

    let mut state = 0; // 0 = prompt, 1 = negative prompt

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.starts_with("Negative prompt:") {
            state = 1;
            let first_neg = trimmed.trim_start_matches("Negative prompt:").trim();
            if !first_neg.is_empty() {
                neg_prompt_lines.push(first_neg);
            }
        } else if trimmed.starts_with("Steps:")
            || (trimmed.contains("Steps:") && trimmed.contains("Sampler:"))
        {
            params_line = Some(trimmed.to_string());
            break;
        } else if state == 0 {
            prompt_lines.push(trimmed);
        } else {
            neg_prompt_lines.push(trimmed);
        }
    }

    let prompt_joined = prompt_lines.join("\n").trim().to_string();
    let neg_joined = neg_prompt_lines.join("\n").trim().to_string();

    if !prompt_joined.is_empty() {
        ai.prompt = Some(prompt_joined);
    }
    if !neg_joined.is_empty() {
        ai.negative_prompt = Some(neg_joined);
    }

    if let Some(params) = params_line {
        parse_params_key_value(&params, ai);
        return true;
    }

    ai.prompt.is_some()
}

fn parse_params_key_value(params_str: &str, ai: &mut AiMetadata) {
    let mut extra = HashMap::new();
    let mut loras = Vec::new();

    let parts = split_params_robust(params_str);

    for (k, v) in parts {
        let k_lower = k.to_lowercase();
        let k_clean = k_lower.trim();
        let v_clean = v.trim();

        match k_clean {
            "steps" => {
                if let Ok(steps) = v_clean.parse::<u32>() {
                    ai.steps = Some(steps);
                }
            }
            "sampler" => {
                ai.sampler = Some(v_clean.to_string());
            }
            "cfg scale" | "cfg" | "guidance" => {
                if let Ok(cfg) = v_clean.parse::<f64>() {
                    ai.cfg_scale = Some(cfg);
                }
            }
            "seed" => {
                if let Ok(seed) = v_clean.parse::<i64>() {
                    ai.seed = Some(seed);
                }
            }
            "size" => {
                ai.size = Some(v_clean.to_string());
            }
            "model" => {
                ai.model_name = Some(v_clean.to_string());
            }
            "model hash" => {
                ai.model_hash = Some(v_clean.to_string());
            }
            "denoising strength" | "denoise" => {
                if let Ok(denoise) = v_clean.parse::<f64>() {
                    ai.denoising_strength = Some(denoise);
                }
            }
            "clip skip" => {
                if let Ok(skip) = v_clean.parse::<u32>() {
                    ai.clip_skip = Some(skip);
                }
            }
            _ => {
                if k_clean.starts_with("lora") || v_clean.contains("<lora:") {
                    loras.push(format!("{}: {}", k, v_clean));
                }
                extra.insert(k, v_clean.to_string());
            }
        }
    }

    ai.extra_params = extra;
    ai.lora_tags = loras;
}

fn split_params_robust(input: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;

    for c in input.chars() {
        if c == '"' {
            in_quote = !in_quote;
            current.push(c);
        } else if c == ',' && !in_quote {
            if let Some((k, v)) = parse_kv_entry(&current) {
                results.push((k, v));
            }
            current.clear();
        } else {
            current.push(c);
        }
    }

    if !current.trim().is_empty() {
        if let Some((k, v)) = parse_kv_entry(&current) {
            results.push((k, v));
        }
    }

    results
}

fn parse_kv_entry(entry: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = entry.splitn(2, ':').collect();
    if parts.len() == 2 {
        let key = parts[0].trim().to_string();
        let val = parts[1].trim().to_string();
        if !key.is_empty() && !val.is_empty() {
            return Some((key, val));
        }
    }
    None
}

/// Parses ComfyUI node graph JSON structure and resolves positive/negative conditioning links.
fn parse_comfyui_json(json_str: &str, ai: &mut AiMetadata) -> bool {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) else {
        return false;
    };

    let Some(map) = val.as_object() else {
        return false;
    };

    let mut positive_prompt_node_ids = Vec::new();
    let mut negative_prompt_node_ids = Vec::new();
    let mut models = Vec::new();
    let mut loras = Vec::new();
    let mut text_nodes: HashMap<String, String> = HashMap::new();

    // First pass: locate KSampler and collect node connections
    for (node_id, node) in map {
        let class_type = node
            .get("class_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let inputs = node.get("inputs");

        if class_type == "CLIPTextEncode" || class_type == "CLIPTextEncodeSDXL" {
            if let Some(inp) = inputs {
                if let Some(text) = inp.get("text").and_then(|t| t.as_str()) {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        text_nodes.insert(node_id.clone(), trimmed.to_string());
                    }
                }
            }
        } else if class_type.contains("KSampler") {
            if let Some(inp) = inputs {
                // Link detection for positive and negative conditioning
                if let Some(pos_link) = inp.get("positive").and_then(|p| p.as_array()) {
                    if let Some(id_val) = pos_link.first() {
                        if let Some(id_str) = id_val.as_str() {
                            positive_prompt_node_ids.push(id_str.to_string());
                        } else if let Some(id_num) = id_val.as_i64() {
                            positive_prompt_node_ids.push(id_num.to_string());
                        }
                    }
                }
                if let Some(neg_link) = inp.get("negative").and_then(|n| n.as_array()) {
                    if let Some(id_val) = neg_link.first() {
                        if let Some(id_str) = id_val.as_str() {
                            negative_prompt_node_ids.push(id_str.to_string());
                        } else if let Some(id_num) = id_val.as_i64() {
                            negative_prompt_node_ids.push(id_num.to_string());
                        }
                    }
                }

                if let Some(seed) = inp.get("seed").and_then(|s| s.as_i64()) {
                    ai.seed = Some(seed);
                } else if let Some(noise_seed) = inp.get("noise_seed").and_then(|s| s.as_i64()) {
                    ai.seed = Some(noise_seed);
                }

                if let Some(steps) = inp.get("steps").and_then(|s| s.as_u64()) {
                    ai.steps = Some(steps as u32);
                }
                if let Some(cfg) = inp.get("cfg").and_then(|c| c.as_f64()) {
                    ai.cfg_scale = Some(cfg);
                }
                if let Some(sampler_name) = inp.get("sampler_name").and_then(|s| s.as_str()) {
                    let scheduler = inp.get("scheduler").and_then(|s| s.as_str()).unwrap_or("");
                    if !scheduler.is_empty() {
                        ai.sampler = Some(format!("{} ({})", sampler_name, scheduler));
                    } else {
                        ai.sampler = Some(sampler_name.to_string());
                    }
                }
                if let Some(denoise) = inp.get("denoise").and_then(|d| d.as_f64()) {
                    ai.denoising_strength = Some(denoise);
                }
            }
        } else if class_type.contains("CheckpointLoader") {
            if let Some(inp) = inputs {
                if let Some(ckpt) = inp.get("ckpt_name").and_then(|c| c.as_str()) {
                    models.push(ckpt.to_string());
                }
            }
        } else if class_type.contains("LoraLoader") {
            if let Some(inp) = inputs {
                if let Some(lora) = inp.get("lora_name").and_then(|l| l.as_str()) {
                    loras.push(lora.to_string());
                }
            }
        } else if class_type == "EmptyLatentImage" {
            if let Some(inp) = inputs {
                let w = inp.get("width").and_then(|v| v.as_u64()).unwrap_or(0);
                let h = inp.get("height").and_then(|v| v.as_u64()).unwrap_or(0);
                if w > 0 && h > 0 {
                    ai.size = Some(format!("{}x{}", w, h));
                }
            }
        }
    }

    // Resolve positive and negative prompts via links or heuristics
    let mut pos_prompts = Vec::new();
    let mut neg_prompts = Vec::new();

    for (node_id, text) in &text_nodes {
        if positive_prompt_node_ids.contains(node_id) {
            pos_prompts.push(text.clone());
        } else if negative_prompt_node_ids.contains(node_id) {
            neg_prompts.push(text.clone());
        } else {
            // Heuristic fallback
            let lower = text.to_lowercase();
            if lower.contains("bad anatomy")
                || lower.contains("watermark")
                || lower.contains("deformed")
                || lower.contains("worst quality")
                || lower.contains("low quality")
            {
                neg_prompts.push(text.clone());
            } else {
                pos_prompts.push(text.clone());
            }
        }
    }

    if !pos_prompts.is_empty() {
        ai.prompt = Some(pos_prompts.join("\n\n"));
    }
    if !neg_prompts.is_empty() {
        ai.negative_prompt = Some(neg_prompts.join("\n\n"));
    }
    if !models.is_empty() {
        ai.model_name = Some(models.join(", "));
    }
    if !loras.is_empty() {
        ai.lora_tags = loras;
    }

    true
}

/// Parses SwarmUI JSON metadata.
fn parse_swarmui_json(json_str: &str, ai: &mut AiMetadata) -> bool {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) else {
        return false;
    };

    let Some(obj) = val.get("sui_image_params").or(Some(&val)) else {
        return false;
    };

    if let Some(prompt) = obj.get("prompt").and_then(|v| v.as_str()) {
        ai.prompt = Some(prompt.to_string());
    }
    if let Some(neg) = obj.get("negativeprompt").and_then(|v| v.as_str()) {
        ai.negative_prompt = Some(neg.to_string());
    }
    if let Some(steps) = obj.get("steps").and_then(|v| v.as_u64()) {
        ai.steps = Some(steps as u32);
    }
    if let Some(cfg) = obj.get("cfgscale").and_then(|v| v.as_f64()) {
        ai.cfg_scale = Some(cfg);
    }
    if let Some(seed) = obj.get("seed").and_then(|v| v.as_i64()) {
        ai.seed = Some(seed);
    }
    if let Some(model) = obj.get("model").and_then(|v| v.as_str()) {
        ai.model_name = Some(model.to_string());
    }

    ai.prompt.is_some()
}

/// Parses InvokeAI JSON metadata.
fn parse_invokeai_json(json_str: &str, ai: &mut AiMetadata) -> bool {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) else {
        return false;
    };

    if let Some(prompt) = val.get("positive_prompt").and_then(|v| v.as_str()) {
        ai.prompt = Some(prompt.to_string());
    }
    if let Some(neg) = val.get("negative_prompt").and_then(|v| v.as_str()) {
        ai.negative_prompt = Some(neg.to_string());
    }
    if let Some(steps) = val.get("steps").and_then(|v| v.as_u64()) {
        ai.steps = Some(steps as u32);
    }
    if let Some(cfg) = val.get("cfg_scale").and_then(|v| v.as_f64()) {
        ai.cfg_scale = Some(cfg);
    }
    if let Some(seed) = val.get("seed").and_then(|v| v.as_i64()) {
        ai.seed = Some(seed);
    }
    if let Some(model) = val
        .get("model")
        .and_then(|v| v.get("name"))
        .and_then(|n| n.as_str())
    {
        ai.model_name = Some(model.to_string());
    }

    ai.prompt.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_a1111_parameters() {
        let text = "a photorealistic cyberpunk girl in neon rain, masterpiece, 8k\nNegative prompt: worst quality, low quality, bad anatomy\nSteps: 28, Sampler: DPM++ 2M Karras, CFG scale: 7.5, Seed: 987654321, Size: 832x1216, Model: dreamshaperXL, Model hash: e6517de7c0, Clip skip: 2, Lora hashes: \"cyber_lora: 1a2b\"";

        let chunks = vec![RawChunkInfo {
            key: "parameters".to_string(),
            value: text.to_string(),
            chunk_type: "tEXt".to_string(),
        }];

        let ai = parse_ai_metadata(&chunks, None).expect("Should detect AI metadata");
        assert_eq!(
            ai.platform.as_deref(),
            Some("Stable Diffusion (A1111 / WebUI / Forge)")
        );
        assert_eq!(
            ai.prompt.as_deref(),
            Some("a photorealistic cyberpunk girl in neon rain, masterpiece, 8k")
        );
        assert_eq!(
            ai.negative_prompt.as_deref(),
            Some("worst quality, low quality, bad anatomy")
        );
        assert_eq!(ai.steps, Some(28));
        assert_eq!(ai.sampler.as_deref(), Some("DPM++ 2M Karras"));
        assert_eq!(ai.cfg_scale, Some(7.5));
        assert_eq!(ai.seed, Some(987654321));
        assert_eq!(ai.size.as_deref(), Some("832x1216"));
        assert_eq!(ai.model_name.as_deref(), Some("dreamshaperXL"));
        assert_eq!(ai.model_hash.as_deref(), Some("e6517de7c0"));
        assert_eq!(ai.clip_skip, Some(2));
    }

    #[test]
    fn test_parse_comfyui_graph() {
        let json = r#"{
            "3": {
                "class_type": "KSampler",
                "inputs": {
                    "cfg": 8,
                    "denoise": 1,
                    "model": ["4", 0],
                    "negative": ["7", 0],
                    "positive": ["6", 0],
                    "sampler_name": "euler",
                    "scheduler": "normal",
                    "seed": 11223344,
                    "steps": 20
                }
            },
            "4": {
                "class_type": "CheckpointLoaderSimple",
                "inputs": { "ckpt_name": "v1-5-pruned-emaonly.safetensors" }
            },
            "5": {
                "class_type": "EmptyLatentImage",
                "inputs": { "batch_size": 1, "height": 512, "width": 512 }
            },
            "6": {
                "class_type": "CLIPTextEncode",
                "inputs": { "text": "an astronaut riding a green horse on Mars" }
            },
            "7": {
                "class_type": "CLIPTextEncode",
                "inputs": { "text": "blurry, dark, low resolution" }
            }
        }"#;

        let chunks = vec![RawChunkInfo {
            key: "prompt".to_string(),
            value: json.to_string(),
            chunk_type: "tEXt".to_string(),
        }];

        let ai = parse_ai_metadata(&chunks, None).expect("Should detect ComfyUI");
        assert_eq!(ai.platform.as_deref(), Some("ComfyUI"));
        assert_eq!(
            ai.prompt.as_deref(),
            Some("an astronaut riding a green horse on Mars")
        );
        assert_eq!(
            ai.negative_prompt.as_deref(),
            Some("blurry, dark, low resolution")
        );
        assert_eq!(ai.steps, Some(20));
        assert_eq!(ai.seed, Some(11223344));
        assert_eq!(ai.sampler.as_deref(), Some("euler (normal)"));
        assert_eq!(
            ai.model_name.as_deref(),
            Some("v1-5-pruned-emaonly.safetensors")
        );
        assert_eq!(ai.size.as_deref(), Some("512x512"));
    }

    #[test]
    fn test_parse_novelai() {
        let json = r#"{"prompt": "masterpiece, 1girl, cat ears", "uc": "nsfw, lowres", "steps": 28, "scale": 6.0, "seed": 456789, "sampler": "k_euler"}"#;
        let chunks = vec![RawChunkInfo {
            key: "Comment".to_string(),
            value: json.to_string(),
            chunk_type: "tEXt".to_string(),
        }];

        let ai = parse_ai_metadata(&chunks, None).expect("Should detect NovelAI");
        assert_eq!(ai.platform.as_deref(), Some("NovelAI"));
        assert_eq!(ai.prompt.as_deref(), Some("masterpiece, 1girl, cat ears"));
        assert_eq!(ai.negative_prompt.as_deref(), Some("nsfw, lowres"));
        assert_eq!(ai.steps, Some(28));
        assert_eq!(ai.cfg_scale, Some(6.0));
        assert_eq!(ai.seed, Some(456789));
        assert_eq!(ai.sampler.as_deref(), Some("k_euler"));
    }
}
