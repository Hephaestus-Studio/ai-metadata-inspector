import init, { inspect_image_wasm, clean_image_wasm } from '@/wasm/core.js'
import wasmUrl from '@/wasm/core_bg.wasm?url'
import type {
  CleanOptions,
  CleanResult,
  ImageMetadataReport,
} from '@/types/metadata'

let isInitialized = false
let initPromise: Promise<unknown> | null = null

/**
 * Initializes the WebAssembly core module.
 * Safe to call multiple times; executes instantiation only once.
 */
export async function initWasm(): Promise<void> {
  if (isInitialized) return

  if (!initPromise) {
    initPromise = init({ module_or_path: wasmUrl })
  }

  await initPromise
  isInitialized = true
}

/**
 * Inspects image bytes or a File object and returns a comprehensive metadata report.
 *
 * @param input - Binary `Uint8Array` or browser `File` object
 * @returns Parsed `ImageMetadataReport`
 */
export async function inspectImage(
  input: File | Uint8Array,
): Promise<ImageMetadataReport> {
  await initWasm()
  const bytes =
    input instanceof Uint8Array
      ? input
      : new Uint8Array(await input.arrayBuffer())

  const report = inspect_image_wasm(bytes) as ImageMetadataReport
  return report
}

/**
 * Sanitizes image metadata losslessly according to provided options.
 *
 * @param input - Binary `Uint8Array` or browser `File` object
 * @param options - Custom `CleanOptions` or default if omitted
 * @returns `CleanResult` with sanitized binary `Uint8Array`
 */
export async function cleanImage(
  input: File | Uint8Array,
  options?: Partial<CleanOptions>,
): Promise<CleanResult> {
  await initWasm()
  const bytes =
    input instanceof Uint8Array
      ? input
      : new Uint8Array(await input.arrayBuffer())

  const result = clean_image_wasm(bytes, options) as CleanResult
  return result
}
