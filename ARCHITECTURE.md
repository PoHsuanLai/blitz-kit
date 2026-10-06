# Architecture

blitz-kit holds the portable repairs and helpers that every Blitz consumer needs: hover sync,
the local net provider, shared fonts, GPU adapter choice, transform-aware hit testing and painted
rectangles, pixel snap, and the scroll engine of design/11. shell-host (Wayland) and quire's `ds-blitz` (design system) both
consume it; each of these was once written twice. It knows no Wayland, no Dioxus and no design
system, and takes and returns CSS px `f64` in Blitz's own units. `CONVENTIONS.md` holds the rules
every repo shares; where the two disagree, this file wins for this repo.

## 1. Crate map

One crate, `blitz-kit` (`crates/blitz-kit`). Allowed dependencies: `blitz-dom`, `blitz-traits`,
`parley`, `keyboard-types` (`scroll::keys` names the keys a scroll answers to), `tokio` (`net` reads
files on the caller's blocking pool), and, behind feature `adapter`, `wgpu` and `serde`
(`AdapterPref` is a shell's serialized setting), behind feature `settings`, `serde` (the scroll
settings read as a settings file's `[scroll]` table).
`scripts/check-boundary.sh` forbids every `ds*`, `dioxus*`, `wayland*`, `smithay*` and `cosmic*`
crate, quire, sill, shell-host and palmrest. Consumers depend on it by path.

| Module | Holds |
|---|---|
| `units` | `Bounds`, `PagePoint` (CSS px `f64`), `Scale120` (integer 120ths) |
| `hover` | the repair for hover changes a resolve makes on its own |
| `net` | `LocalNet`, a `NetProvider` for `data:` and absolute `file:` only; `LocalSource::read`, the same reads on the calling thread |
| `data_url` | `decode`: a `data:` URL's bytes (percent, base64, URL-safe base64) |
| `fonts` | `SharedFonts`, `FontFaces`: one `FontContext` for every document |
| `adapter` (feature) | `AdapterFacts`, `DeviceKind`, `GpuBackend`, `AdapterPref`, `PciVendor`, `PciDevice`, `rank`; `ranked`, `request_device`, `block_on`, `ADAPTER_ENV` |
| `hit` | the element Blitz would hit at a point, lifted to its element |
| `paint_rect` | where a laid-out box paints, through its own and its ancestors' transforms |
| `snap` | `snap_layout`: whole device pixels for every box after a resolve |
| `element_id` | `ElementId`: an element's `id` attribute, as a typed name |
| `scroll` | the scroll engine (design/11), shared by every host: its physics, the document side, and a driver joining them (below) |

## 2. Public API

| Module | Names |
|---|---|
| `units` | `Bounds`, `PagePoint`, `Scale120::{factor, is_whole}` |
| `hover` | `LastMove::{Unknown, At}`, `remember`, `Shift<N>`, `HoverSync::{Unchanged, Restore, Clear}`, `decide`, `probe_points`, `repair(&mut dyn Document, &LastMove, Shift<NodeId>) -> Repaired::{Yes, No}` |
| `net` | `LocalNet`, `LocalSource::{Data, File, Unservable}`, `LocalSource::{of, read}` |
| `data_url` | `decode` |
| `fonts` | `SharedFonts::{system, system_with, bundled, register, for_document}`, `FontFaces` |
| `adapter` | `AdapterFacts::{of, label}`, `DeviceKind`, `GpuBackend`, `AdapterPref::{with_env, Device}`, `PciVendor`, `PciDevice`, `rank`, `ranked`, `request_device`, `block_on`, `ADAPTER_ENV` |
| `hit` | `element_at`, `element_of`, `is_content_element` |
| `paint_rect` | `Affine2`, `Placed`, `painted_bounds`, `painted_rect` |
| `snap` | `snap_layout(&mut BaseDocument, Scale120)` |

`units::Scale120::from_factor` turns a viewport's `f64` scale into 120ths.

### `scroll`

One engine for every Blitz host: shell-host's surfaces and the app windows of quire's `ds-blitz`
both run it, so a wheel detent, a touchpad flick and an arrow key move the same way everywhere
(design/11-BEHAVIOUR-scroll.md). A host adds only the translation from its own device events, its
clock and its frame requests.

| Part | Modules and names |
|---|---|
| Pure physics, table-tested, time passed in | `config` (`ScrollSettings` and its value types; serde with feature `settings`), `geom` (`Px`, `ScrollAxis`, `Dir`, `Scroller`, `Geom`, `Latch`, `Area`, `ViewPoint`), `smooth` (the 1000 px/s, 200 ms, `--e-out` step), `velocity` (release velocity), `momentum` (the glide), `rubber` (stretch and snap-back), `keys` (`ScrollKey`, `scroll_key`, `page`, the held arrow's ramp and spring), `latch` (`latch`, `latch_rigid`, `AxisLock`, `WheelBurst`), `pad` (`PointerScroll`, `Pad`, `PadEvent`: a touchpad gesture from pans and a stop), `engine` (`Engine`, `ScrollIn`, `ScrollOut`, `Physics`, `step`), `target` (`Target`: the smooth step for a consumer that owns its offset), `cmd` (`ScrollCmd`, `into_view`), `time` (`Elapsed`), `tuning` (`Tuning`, `Env`) |
| The document side | `doc` (`chain_at`, `geom`, `write`, `wheel_route`, `key_focus`, `scroller_by_id`, `container_of`, `area`, `WheelRoute`, `KeyFocus`): the scrollers under a point, their geometry, raw offset writes and the `data-wheel`, `data-overscroll` and `data-keys` markers |
| The driver | `driver::ScrollDriver::{run, frame, pointer, detents, begin, key, key_up, command, motion}`, `Moves`, `Moved`, `Claim`, `KeyUse`, `KeyRepeat`: the engine applied to one document |

Ownership (design/11 §11.3.1): the host does not let Blitz's own scroll act; it writes raw offsets
every frame and asks for frames while `ScrollDriver::motion` says `Animating`. An element marked
`data-wheel="capture"` gets the raw wheel and the engine leaves it alone (`doc::wheel_route`). The
kit never reads a clock: every call takes an `Elapsed` on a timeline the host chooses.

`hover::repair` takes any `Document` so a Dioxus document dispatches the re-fed move into its
components; it reads the hover state and never the layout. Tests for each module run against a
real `BaseDocument` built by hand (`tests/support`), so a repair is tested once for every
consumer.

## 3. Repo rules

- Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D
  warnings`, `cargo test --workspace --all-features`, `./scripts/check-boundary.sh`,
  `cargo deny check licenses`.
- The pinned dependency block in the root `Cargo.toml` is copied verbatim from shell-host's
  (source of truth: quire's `docs/workspace-deps.toml`); the kit never bumps it alone.
- No `unsafe`, no global state, no reading the environment: every function takes what it needs.
- A helper that needs Wayland, Dioxus or a design-system type does not belong here; a repair a
  consumer keeps its own copy of does.
