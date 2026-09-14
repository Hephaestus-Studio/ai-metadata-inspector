# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

### Features

- *(core)* Initialize Rust core crate with metadata types and rustdoc - ([56cbdf5](https://github.com/hephaestus-studio/ai-metadata-inspector/commit/56cbdf5a6a695b9fb28bc6c413010ff724acd807))
- *(core)* Implement PNG parser, metadata extractor, and lossless cleaner - ([21b8c6a](https://github.com/hephaestus-studio/ai-metadata-inspector/commit/21b8c6a5ab2ef03fdc055a6b848259017e6f9f35))
- *(core)* Implement JPEG segment parser, metadata extractor, and lossless cleaner - ([23b9dcf](https://github.com/hephaestus-studio/ai-metadata-inspector/commit/23b9dcfac4a9a82c99d34d775182ac677208febe))
- *(core)* Implement WebP container parser, metadata extractor, and lossless cleaner - ([dd3b548](https://github.com/hephaestus-studio/ai-metadata-inspector/commit/dd3b54821ebd0c2165d9d9ff309c24d8b4b47b5d))
- *(core)* Implement C2PA provenance and JUMBF box detector - ([2734c82](https://github.com/hephaestus-studio/ai-metadata-inspector/commit/2734c821d57e2a38b7cf6afeb8b6469e1dbf6171))
- *(core)* Implement generative AI metadata parser for SD, ComfyUI, NovelAI, SwarmUI, and InvokeAI - ([ec56918](https://github.com/hephaestus-studio/ai-metadata-inspector/commit/ec569182603aec4c4d346ad7b8fa1b2950189103))
- *(core)* Implement core inspection pipeline, WASM bindings, and risk assessment - ([d4ccb3d](https://github.com/hephaestus-studio/ai-metadata-inspector/commit/d4ccb3d6297808ce66cdc39af092de14cee2593f))
- *(frontend)* Integrate WASM service, define TypeScript types, and configure path aliases - ([03c0917](https://github.com/hephaestus-studio/ai-metadata-inspector/commit/03c0917a70bb536176c46bb35442dee03e3e71f4))
- *(ui)* Redesign ergonomic studio workspace with unified header dock, segmented tabs and color harmony - ([22f1069](https://github.com/hephaestus-studio/ai-metadata-inspector/commit/22f1069c43cc89bb152783d9d806fb55fd35bdee))

### Maintenance

- Init project - ([1864997](https://github.com/hephaestus-studio/ai-metadata-inspector/commit/1864997fa436a2ee915af3183a96628038737fb2))
- Configure GitHub Pages deployment and vite base path - ([ebef014](https://github.com/hephaestus-studio/ai-metadata-inspector/commit/ebef014e4915b782e69375153a2735bffb66f984))
- Add prettier and configure format scripts - ([13cd80d](https://github.com/hephaestus-studio/ai-metadata-inspector/commit/13cd80d0a69920e79086e6e786da44d947d36444))
- Setup git-cliff and automated changelog workflow - ([a9d46d9](https://github.com/hephaestus-studio/ai-metadata-inspector/commit/a9d46d904c3e2e5f24dee4fcf8c860857a787aaa))
- Specify pnpm version 11 in deploy-pages workflow - ([7485fd0](https://github.com/hephaestus-studio/ai-metadata-inspector/commit/7485fd06338f8d00d6a894dcd9abff32a0b6a8eb))
