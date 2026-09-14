/**
 * Utility to generate valid demo images in memory with realistic metadata
 * for instant interactive testing without downloading external files.
 */

function crc32(bytes: Uint8Array): number {
  let c = 0xffffffff
  for (let i = 0; i < bytes.length; i++) {
    c = (c >>> 8) ^ CRC_TABLE[(c ^ bytes[i]) & 0xff]
  }
  return (c ^ 0xffffffff) >>> 0
}

const CRC_TABLE = new Uint32Array(256)
for (let n = 0; n < 256; n++) {
  let c = n
  for (let k = 0; k < 8; k++) {
    c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1
  }
  CRC_TABLE[n] = c
}

function createPngWithChunks(
  chunks: { type: string; data: Uint8Array }[],
): Uint8Array {
  const parts: Uint8Array[] = []
  // PNG Magic Header
  parts.push(new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]))

  // IHDR chunk (1x1 8-bit RGBA)
  const ihdrData = new Uint8Array([
    0x00,
    0x00,
    0x03,
    0x40, // width: 832
    0x00,
    0x00,
    0x04,
    0xc0, // height: 1216
    0x08,
    0x06,
    0x00,
    0x00,
    0x00,
  ])
  appendPngChunk(parts, 'IHDR', ihdrData)

  for (const chunk of chunks) {
    appendPngChunk(parts, chunk.type, chunk.data)
  }

  // IDAT minimal compressed pixel data
  const idatData = new Uint8Array([
    0x78, 0x9c, 0x63, 0x60, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01,
  ])
  appendPngChunk(parts, 'IDAT', idatData)

  // IEND chunk
  appendPngChunk(parts, 'IEND', new Uint8Array([]))

  let totalLen = 0
  for (const p of parts) totalLen += p.length
  const out = new Uint8Array(totalLen)
  let offset = 0
  for (const p of parts) {
    out.set(p, offset)
    offset += p.length
  }
  return out
}

function appendPngChunk(
  parts: Uint8Array[],
  type: string,
  data: Uint8Array,
): void {
  const len = data.length
  const header = new Uint8Array(8)
  // Length (big endian)
  header[0] = (len >>> 24) & 0xff
  header[1] = (len >>> 16) & 0xff
  header[2] = (len >>> 8) & 0xff
  header[3] = len & 0xff
  // Type
  for (let i = 0; i < 4; i++) header[4 + i] = type.charCodeAt(i)

  // Calculate CRC over type + data
  const crcTarget = new Uint8Array(4 + len)
  crcTarget.set(header.subarray(4, 8), 0)
  crcTarget.set(data, 4)
  const chunkCrc = crc32(crcTarget)

  const footer = new Uint8Array(4)
  footer[0] = (chunkCrc >>> 24) & 0xff
  footer[1] = (chunkCrc >>> 16) & 0xff
  footer[2] = (chunkCrc >>> 8) & 0xff
  footer[3] = chunkCrc & 0xff

  parts.push(header, data, footer)
}

function createTextChunk(key: string, value: string): Uint8Array {
  const enc = new TextEncoder()
  const keyBytes = enc.encode(key)
  const valBytes = enc.encode(value)
  const data = new Uint8Array(keyBytes.length + 1 + valBytes.length)
  data.set(keyBytes, 0)
  data[keyBytes.length] = 0 // null separator
  data.set(valBytes, keyBytes.length + 1)
  return data
}

/**
 * Generates a sample Stable Diffusion (A1111 / WebUI) PNG.
 */
export function generateSampleSdWebUi(): File {
  const sdParams = `masterpiece, best quality, ultra-detailed, 1girl, cyberpunk samurai aesthetic, neon lighting, highly detailed katana, volumetric fog, cinematic lighting, 8k resolution, raytracing
Negative prompt: worst quality, low quality, normal quality, lowres, bad anatomy, bad hands, missing fingers, extra digits, text, error, signature, watermark, username, blurry
Steps: 30, Sampler: DPM++ 2M Karras, CFG scale: 7.5, Seed: 3847291048, Size: 832x1216, Model: dreamshaperXL_v21TurboDPMS, Model hash: e6517de7c0, Denoising strength: 0.7, Clip skip: 2, Lora hashes: "cyber_armor_v2: 1a2b3c4d, neon_fx: 9f8e7d6c"`

  const chunk = {
    type: 'tEXt',
    data: createTextChunk('parameters', sdParams),
  }
  const bytes = createPngWithChunks([chunk])
  return new File([bytes.buffer as ArrayBuffer], 'sample_sdxl_cyberpunk.png', {
    type: 'image/png',
  })
}

/**
 * Generates a sample ComfyUI Node Graph Workflow PNG.
 */
export function generateSampleComfyUi(): File {
  const comfyPrompt = JSON.stringify(
    {
      '3': {
        class_type: 'KSampler',
        inputs: {
          cfg: 8.0,
          denoise: 1.0,
          model: ['4', 0],
          negative: ['7', 0],
          positive: ['6', 0],
          sampler_name: 'euler_ancestral',
          scheduler: 'karras',
          seed: 9948271635,
          steps: 25,
        },
      },
      '4': {
        class_type: 'CheckpointLoaderSimple',
        inputs: {
          ckpt_name: 'flux1-dev-fp8.safetensors',
        },
      },
      '5': {
        class_type: 'EmptyLatentImage',
        inputs: {
          batch_size: 1,
          height: 1024,
          width: 1024,
        },
      },
      '6': {
        class_type: 'CLIPTextEncode',
        inputs: {
          text: 'A majestic celestial dragon coiled around an ancient observatory under a starry nebula, hyper-detailed digital art, octane render',
        },
      },
      '7': {
        class_type: 'CLIPTextEncode',
        inputs: {
          text: 'blurry, oversaturated, low quality, watermark, bad anatomy, noisy',
        },
      },
      '10': {
        class_type: 'LoraLoader',
        inputs: {
          lora_name: 'CelestialGlow_Flux.safetensors',
          strength_model: 0.85,
        },
      },
    },
    null,
    2,
  )

  const chunks = [
    { type: 'tEXt', data: createTextChunk('prompt', comfyPrompt) },
    {
      type: 'tEXt',
      data: createTextChunk(
        'workflow',
        JSON.stringify({ nodes: [], links: [] }),
      ),
    },
  ]
  const bytes = createPngWithChunks(chunks)
  return new File(
    [bytes.buffer as ArrayBuffer],
    'sample_comfyui_flux_dragon.png',
    {
      type: 'image/png',
    },
  )
}

/**
 * Generates a sample Smartphone JPEG with EXIF and GPS coordinates.
 */
export function generateSampleGpsPhoto(): File {
  // Minimal valid JPEG with APP1 EXIF segment containing GPS tags (San Francisco coordinates)
  const exifSegment = new Uint8Array([
    // APP1 Header
    0xff,
    0xe1,
    0x00,
    0x88, // Length: 136 bytes
    0x45,
    0x78,
    0x69,
    0x66,
    0x00,
    0x00, // "Exif\0\0"
    // TIFF Header (Little Endian: "II*\0")
    0x49,
    0x49,
    0x2a,
    0x00,
    0x08,
    0x00,
    0x00,
    0x00, // 8 offset to 0th IFD
    // 0th IFD: 2 entries
    0x02,
    0x00,
    // Tag 1: Make ("Apple")
    0x0f,
    0x01,
    0x02,
    0x00,
    0x06,
    0x00,
    0x00,
    0x00,
    0x26,
    0x00,
    0x00,
    0x00,
    // Tag 2: GPS IFD Pointer -> offset 0x30
    0x25,
    0x88,
    0x04,
    0x00,
    0x01,
    0x00,
    0x00,
    0x00,
    0x30,
    0x00,
    0x00,
    0x00,
    0x00,
    0x00,
    0x00,
    0x00, // Next IFD (none)
    // String "Apple\0" at offset 0x26 (0x26 from TIFF header = 0x26 - 8 = offset 38)
    0x41,
    0x70,
    0x70,
    0x6c,
    0x65,
    0x00,
    0x00,
    0x00,
    // GPS IFD at offset 0x30 (48): 4 entries (GPSLatitudeRef, GPSLatitude, GPSLongitudeRef, GPSLongitude)
    0x04,
    0x00,
    // GPSLatRef 'N'
    0x01,
    0x00,
    0x02,
    0x00,
    0x02,
    0x00,
    0x00,
    0x00,
    0x4e,
    0x00,
    0x00,
    0x00,
    // GPSLatitude (3 rationals: 37/1, 46/1, 297/10 -> 37.774917 N) -> offset 0x66
    0x02,
    0x00,
    0x05,
    0x00,
    0x03,
    0x00,
    0x00,
    0x00,
    0x66,
    0x00,
    0x00,
    0x00,
    // GPSLonRef 'W'
    0x03,
    0x00,
    0x02,
    0x00,
    0x02,
    0x00,
    0x00,
    0x00,
    0x57,
    0x00,
    0x00,
    0x00,
    // GPSLongitude (3 rationals: 122/1, 25/1, 98/10 -> 122.4194 W) -> offset 0x7e
    0x04,
    0x00,
    0x05,
    0x00,
    0x03,
    0x00,
    0x00,
    0x00,
    0x7e,
    0x00,
    0x00,
    0x00,
    0x00,
    0x00,
    0x00,
    0x00,
    // Rational values for Lat: 37/1, 46/1, 297/10
    0x25,
    0x00,
    0x00,
    0x00,
    0x01,
    0x00,
    0x00,
    0x00,
    0x2e,
    0x00,
    0x00,
    0x00,
    0x01,
    0x00,
    0x00,
    0x00,
    0x29,
    0x01,
    0x00,
    0x00,
    0x0a,
    0x00,
    0x00,
    0x00,
    // Rational values for Lon: 122/1, 25/1, 98/10
    0x7a,
    0x00,
    0x00,
    0x00,
    0x01,
    0x00,
    0x00,
    0x00,
    0x19,
    0x00,
    0x00,
    0x00,
    0x01,
    0x00,
    0x00,
    0x00,
    0x62,
    0x00,
    0x00,
    0x00,
    0x0a,
    0x00,
    0x00,
    0x00,
  ])

  const jpegHeader = new Uint8Array([0xff, 0xd8])
  // SOF0 (1920x1080)
  const sof0 = new Uint8Array([
    0xff, 0xc0, 0x00, 0x0b, 0x08, 0x04, 0x38, 0x07, 0x80, 0x01, 0x01, 0x11,
    0x00,
  ])
  // SOS & Scan
  const sos = new Uint8Array([
    0xff, 0xda, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3f, 0x00, 0x12, 0x34,
  ])
  const eoi = new Uint8Array([0xff, 0xd9])

  const totalLen =
    jpegHeader.length +
    exifSegment.length +
    sof0.length +
    sos.length +
    eoi.length
  const out = new Uint8Array(totalLen)
  let offset = 0
  out.set(jpegHeader, offset)
  offset += jpegHeader.length
  out.set(exifSegment, offset)
  offset += exifSegment.length
  out.set(sof0, offset)
  offset += sof0.length
  out.set(sos, offset)
  offset += sos.length
  out.set(eoi, offset)

  return new File(
    [out.buffer as ArrayBuffer],
    'sample_smartphone_gps_photo.jpg',
    {
      type: 'image/jpeg',
    },
  )
}

/**
 * Generates a sample C2PA Content Credentials Verified Image.
 */
export function generateSampleC2pa(): File {
  const c2paManifest = `{"claim_generator": "Adobe Firefly 2.0 (Content Authenticity Initiative)", "issuer": "Adobe CA CAI Root", "dc:title": "AI Generative Fill"}`
  const chunks = [
    { type: 'caPI', data: new TextEncoder().encode(c2paManifest) },
  ]
  const bytes = createPngWithChunks(chunks)
  return new File(
    [bytes.buffer as ArrayBuffer],
    'sample_c2pa_verified_firefly.png',
    {
      type: 'image/png',
    },
  )
}
