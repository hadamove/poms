# Improvements

## Easy, high priority

- Add types to wgsl shaders & refine them
- Extract parser to a completely different module / use some other parser?
  - Allow parsing of larger molecules - different file types ✅
  - Refactor parser so that it uses file streams instead of files (figure out if it's even possible with wasm)
  - Allow fetching of molecules from the internet
- Fix camera movement and make it work across different platforms
- Fix windows build

## Harder, high priority

- Refactor resources & ses-state
- Resolution up to 512^3 => change array buffers to 3d textures
  - Deal with the TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES problem for read_write 3D textures.
    - Use two textures, one for reading one for writing?
- Ambient space occlusion SSAO (use module from github? or implement own)
- Fast distance refinement (chebychev) ✅
  - Fix buggy behvaior ✅
  - Remove the need for gridpoint class buffer -> only use predecessor ✅

## Easy, low priority

- Improve animation UI
- Show SES parameters in the UI (grid offset, grid size, time to render)
- Make repository public on github
- Use macros to learn them where appropriate? e.g. the parser
- Learn how to properly use modules (do not use mod.rs everywhere)

## Harder, low priority

- Anti-aliasing for spacefill
- Figure out transparent rendering?
- Separate calculation from actual rendering + UI -> workspace
  - Also separate general raycasting of a distance field
- Adjust number of workgroups to fps
- Make the grid coarser in the proximity of the camera??
- Better handling and architecture of gui events
