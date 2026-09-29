# What we want from Blitz that it does not do (yet)

A running list for our Blitz fork (`github.com/PoHsuanLai/blitz` rev `bf588142` = upstream
`e99fbdbd` plus one restyle patch) and for upstream PRs to DioxusLabs/blitz, stylo and vello. The
stack at the pin: dioxus-native-dom, stylo 0.21, parley 0.11, anyrender_vello_hybrid 0.10,
anyrender_vello_cpu 0.17. Each entry says what we want, what Blitz does today, how we know, and the
cheapest route. Add entries as work hits them; keep the evidence line honest (`verified` = a test,
a spike or a source line read at the pin; `assumed` = research 2026-09-23, not yet checked here).
Paths are relative to `~` (`quire/FINDINGS.md` and so on).

Routes: **fork** (patch our Blitz fork), **PR** (worth offering upstream DioxusLabs/blitz or
stylo/vello), **kit** (worked around in blitz-kit), **quire** (worked around in the design
system), **host** (worked around in shell-host).

## Paint and effects

| Want | Blitz today | Evidence | Route |
| --- | --- | --- | --- |
| `backdrop-filter: blur()` inside a surface (frosted cards over content in the same window) | Every Vello backend takes it as `_backdrop_filter` and ignores it; only anyrender_skia implements it | verified: quire/FINDINGS.md S15 (cpu and hybrid); Skia claim assumed, research 2026-09-23 | PR (vello backends), quire (compositor blur through `ext-background-effect-v1`, `data-blur` materials) |
| `filter` (`saturate`, `grayscale`, `brightness`, ...) for vibrancy and muted avatars | anyrender_vello_cpu 0.17 drops every filter when `multithreading` is on (our pinned feature); anyrender_vello_hybrid 0.10 returns `None` for every `ColorMatrix` filter and allows one filter node | verified: quire/FINDINGS.md S16 | PR (anyrender_vello_*), quire (colours precomputed in OKLab; lint bans `filter`) |
| `filter: blur()` on hybrid only | Applied by hybrid, dropped by cpu, dropped on plain anyrender_vello; no design may rely on it | verified: quire/FINDINGS.md S16; plain-vello part assumed, research 2026-09-23 | quire (banned) |
| `mix-blend-mode` (grain and lighting overlays) | Not exercised; assumed unsupported | assumed: research 2026-09-23; quire/crates/ds/src/lint/blitz.rs:20 | PR, quire (PNG grain layer, precomputed blends) |
| `text-shadow` | Not exercised; assumed unsupported | assumed: research 2026-09-23; quire/crates/ds/src/lint/blitz.rs:33 | PR, quire (avoided) |
| Sweep (conic) gradients, gradient extend Repeat/Reflect, gradient paint on strokes and text, blur, filters and blend operators beyond source-over in PDF replay | anyrender's recording `Scene` replay (pdfrum) lacks them; anyrender `draw_glyphs` also carries no text, so PDF glyph text is recovered by walking parley layouts | verified: quire/FINDINGS.md:116 ("pdfrum asks"), :25 region ("anyrender draw_glyphs", Open items) | PR (anyrender: text-and-clusters argument), quire |
| vello_hybrid glyph atlas cache on (COLRv1 emoji re-flattened every frame; the 300-emoji grid costs 4-5 ms CPU) | `atlas_cache_enabled: false`, never turned on by anyrender_vello_hybrid | verified: quire/FINDINGS.md:48, table at :989 | PR (anyrender_vello_hybrid), or fork that crate |
| CBDT bitmap emoji on vello_hybrid | glifo's `png` feature paints them on cpu but hybrid panics (`pixmap image sources are not supported by Vello Hybrid`); the feature stays off | verified: quire/FINDINGS.md:944-958 | PR (vello_hybrid), quire (COLRv1 faces only) |
| Blitz to draw a clean fractional-scale hairline | Layout rounds to whole logical px, stylo floors border widths to device px then taffy rounds back up; no CSS value gives a crisp 1 device px line at 1.25 / 1.5 / 1.75 | verified: quire/FINDINGS.md:668-690 ("Pixel snapping") | kit (`snap_layout`), quire (`snap_to_device`), PR (device-pixel layout rounding) |
| Transform performance on many nodes | Transforms on more than about 200 nodes lag (blitz #595) | assumed: research 2026-09-23; quire/design/05-MOTION.md:986 | quire (stagger index capped at 12, lift one node at a time), PR |
| A transformed absolutely positioned box inside an inline box painted correctly | Paint bug (blitz #840) | assumed: research 2026-09-23; quire/design/05-MOTION.md:994 | quire (never transform an absolute box in an inline box), PR |
| A working many-layer document | About sixty tinted cards in one document lose layers (vanish or paint at partial opacity), each row alone is right; not chased | verified: quire/FINDINGS.md:143 | quire (level sheet renders row by row), PR once minimised |
| 3-D transforms | Painted untransformed (and hit-tested that way) | verified: shell-host/FINDINGS.md:329 | none wanted |
| `outline-style` other than solid, native selection colour | Every outline style but `none` paints solid; selection colour is a blitz-paint constant (rgb 180 213 255) no style reaches | verified: quire/FINDINGS.md:376, :399 | quire (dashed look is a border; a Boxed mask covers the selection), PR |
| SVG with its own CSS (animation, `stroke`/`fill` rules on children) | SVG children are rendered from their own attributes only; CSS on `path`/`circle` does nothing | verified: quire/FINDINGS.md S6, "SVG and icons" | quire (`Glyph` writes attributes; stacked parts cross-fade in HTML) |

## Style, selectors and the cascade

| Want | Blitz today | Evidence | Route |
| --- | --- | --- | --- |
| dioxus-native-dom to write attributes with no namespace, so `[data-theme=dark]`, `[aria-selected=true]` and `[data-variant=x]` match | `mutation_writer.rs` creates every attribute with `qual_name(name, None)` in the HTML namespace and stylo's `attr_matches` compares namespaces, so unprefixed attribute selectors never match | verified: quire/FINDINGS.md:27 and S2 (:200) | PR (dioxus-native-dom), quire (every attribute selector is `[*\|name=v]`; lint flags the plain form) |
| Dioxus `open` on `details` to open it (and other boolean attributes to follow the DOM) | The UA rule `details:not([open]) > ...` matches only a null-namespace `open`; author rules cannot beat a UA `!important`; `disabled="false"` counts as disabled | verified: quire/FINDINGS.md:257-268 (:267 boolean) | PR (same fix as above), quire (`TreeItem` writes `open` itself; booleans written only when true) |
| `data-*` names folded to lowercase as in HTML | Names are case-sensitive; `DataName::parse` accepts lowercase only | verified: quire/FINDINGS.md:269 | quire |
| `:focus-visible` and `:focus-within` | stylo.rs hard-codes both to `false`; pointer-down focuses only text inputs and checkboxes | verified: quire/FINDINGS.md:234 (S12) | fork or PR (pass the state through), quire (`data-modality` stamp; lint bans both) |
| `@property` registered custom properties, and animating them (gradient stops, angles) | Not supported; a custom property does not interpolate | assumed: research 2026-09-23 | PR (stylo), quire (Rust-driven tweens write plain `--x`) |
| `text-overflow: ellipsis` | Hard clips with no "..." (blitz #888, PR #893 open) | verified: quire/FINDINGS.md S13 (:236) | PR (help land #893), quire (`.ds-truncate` mask fade, `text::clip_chars`) |
| `line-clamp` / `-webkit-line-clamp` | Not exercised; assumed unsupported | assumed: research 2026-09-23; quire/crates/ds/src/lint/blitz.rs:29 | PR, quire (`max-height` of whole lines, measured fade; `clip_chars`) |
| `position: sticky` | Open upstream | assumed: research 2026-09-23; quire/crates/ds/src/lint/blitz.rs:22 | PR, quire (banned; headers sit outside the scroller) |
| `font-optical-sizing` from the font size | Not set, so Inter's `opsz` is pinned into two families | verified: quire/FINDINGS.md:356 | quire (Inter / Inter Display cuts) |
| Inline boxes with padding, border and horizontal margin; trailing whitespace kept | Text laid out but padding, border and a plain span's margin are dropped; trailing whitespace of an inline element is dropped | verified: quire/FINDINGS.md:371-375 | quire (`inline-block`; spaces written as outside text nodes) |
| Measured `height` transitions (auto to content) | A height that follows content snaps | verified: quire/FINDINGS.md:126 (Pane switcher), :"Motion and timing" | quire (`max-height` in whole lines), PR (stylo/taffy) |
| A user-agent sheet without the browser's `body { margin: 8px }`, `button { justify-content: center }` and the 300 x 150 unsized `input` | Blitz's `DEFAULT_CSS` carries all three | verified: shell-host/FINDINGS.md:101 (F62); quire/FINDINGS.md:279 | host (`BODY_RESET` UA sheet), quire (reset, sized inputs) |
| `html`, `body` and `#main` sized to the viewport | `#main` is `height: auto`; `100%` heights and `position: fixed; inset: 0` resolve against a 0 px parent (taffy places fixed boxes against the parent); `vh` / `vw` work | verified: shell-host/FINDINGS.md:118 (F41); quire/FINDINGS.md:"Root height" | PR (dioxus-native-dom `#main`), host (sized UA root), quire (`RootExtent::Viewport`) |
| Restarting a CSS animation when `display: none` is removed | Stylo does not restart it | verified: quire/FINDINGS.md:61 | quire (`X` / `X--b` alias swap) |
| Restyle of an element whose animation was cancelled mid-way | The cancelling restyle cascades the mid-animation value and nothing restyles again (stylo sets `ElementAnimationSet::dirty`, Blitz never reads it); the stale transform is painted and hit-tested. Fixed in our fork rev `bf588142`, not upstream | verified: sill/FINDINGS.md:231 (G295; `tests/cancelled_animation.rs`); quire/FINDINGS.md:25 | fork (carried), PR (offer the patch) |
| Runtime `animationend` and `transitionend` events (blitz #863) | Never dispatched | assumed: research 2026-09-23; quire/design/05-MOTION.md:875 | PR, quire (Rust timers from the shared timing table) |
| Time source Blitz does not own: double-click interval and scrollbar fade read `Instant` in `pub(crate)` fields | The virtual test clock cannot reach them; two clicks at one spot on a virtual clock are always a double click | verified: quire/FINDINGS.md:56, :828 | PR (inject a clock), quire (wall-clock tests where it matters) |

## Text and editing

| Want | Blitz today | Evidence | Route |
| --- | --- | --- | --- |
| The text editor to honour the input's `font-family`, `font-weight`, `letter-spacing`, `text-align` | `create_text_editor` clears the editor's styles and keeps only size, line height and brush; a centred input draws its value at the left in the default face | verified: quire/FINDINGS.md:58, :382 | fork or PR (blitz-dom `layout/construct.rs`), quire (`text-align: start`, own placeholder face) |
| Password fields that mask | blitz-dom paints `type=password` characters as typed; no masking in blitz-dom or blitz-paint | verified: quire/FINDINGS.md:388 | fork or PR, quire (transparent text plus a drawn mask, drawn caret and selection) |
| Caret and selection style (width, blink, colour) | Caret is 1.5 px, line-box high, never blinks; selection colour is a constant; `caret-color: transparent` is honoured | verified: quire/FINDINGS.md:395-400 | PR, quire |
| A `change` event from text inputs | Only `input`, selection and implicit submit are dispatched | verified: quire/FINDINGS.md:403 | quire (Enter or blur makes it) |
| A file picker | `file-input` feature (off) only draws a button | verified: quire/FINDINGS.md:405 | quire, host (asks the host to pick) |
| IME events to reach a Dioxus handler | blitz-shell converts winit `Ime` to `UiEvent::Ime`, dioxus-native-dom maps it to `None` and its composition converter is `unimplemented!()`; blitz-dom edits only a focused `input` / `textarea` | verified: quire/FINDINGS.md:425-436 | PR (dioxus-native-dom), quire (`ds-native` routes window `Ime` to the edit surface first; recheck at each bump) |
| IME switched on for a custom editor | blitz-dom enables it only when a text field takes focus, and only once the node has its `TextInputData` | verified: quire/FINDINGS.md:435; shell-host/FINDINGS.md:542 (F71) | quire (surface switches it on), host |
| Programmatic focus to dispatch `focus` / `blur` | dioxus-native-dom's `set_focus` has a TODO to queue focus events; the element losing the caret hears no `blur` | verified: quire/FINDINGS.md:39 | PR (dioxus-native-dom), quire (tells the new field itself) |
| Focus on mount for a text field | The editor is built with the document's first layout; select-all and caret writes answer `Busy` until then | verified: quire/FINDINGS.md:414 | quire (retries a frame later) |
| A click on a non-editable target to keep focus; a keyboard-synthesised click | Blitz's click default clears focus; no click from Enter or Space | verified: quire/FINDINGS.md:474, :771 | quire (`HostClickFocus`, kept-click rule; controls handle keys) |
| Dioxus capture phase | dioxus 0.7 has none and Blitz dispatches bottom-up only | verified: quire/FINDINGS.md:492 | quire |
| CJK dictionary line breaking | parley 0.11 prints `ICU4X data error: No segmentation model for complex script` and wraps CJK without word boundaries | verified: quire/FINDINGS.md:45 | PR (parley / ICU data) |
| WOFF2 faces, `unicode-range`, name-table family matching | fontique reads sfnt only, has no `unicode-range`, registers by the family in the file | verified: quire/FINDINGS.md:348 | quire (TTF subsets with rewritten names) |
| A clipboard-less session not to panic | blitz-shell unwraps `arboard::Clipboard::new()` | verified: quire/FINDINGS.md:44 | PR (blitz-shell), quire (own calls catch it) |
| Native tooltips for `title=` | Blitz draws none | verified: quire/FINDINGS.md:79 | quire (`Tooltip`, `HoverTarget`) |

## Input, hover and hit testing

| Want | Blitz today | Evidence | Route |
| --- | --- | --- | --- |
| Hover re-resolved (with `pointerenter` / `pointerleave`) when content moves under a still pointer | `resolve` ends with `refresh_hover`, which moves the hover silently (its own TODO); the next move's diff is empty, so the arriving element is never entered and the left one never hears leave | verified: shell-host/FINDINGS.md:486 (F59); quire/FINDINGS.md:744; blitz-kit `tests/hover_sync.rs` | kit (`hover::repair`), PR (dispatch from `refresh_hover`) |
| `clear_hover()` to dispatch `mouseleave` | Unhovers without dispatching | verified: shell-host/FINDINGS.md:484 | host (pointer leave is a move to (-1, -1)) |
| Hit testing that returns an atomic inline (`inline-flex` button) alone in an inline formatting context | `Node::hit` returns the parent | verified: quire/FINDINGS.md:33; ignored repro `quire/crates/ds-native/tests/click.rs` | PR, quire (buttons live in flex rows) |
| CSS 2.1 Appendix E order for `z-index: auto` positioned boxes across the stacking context | Ranked per parent: an absolute card without z-index after `relative` rows goes under them | verified: quire/FINDINGS.md:33; test `blitz_orders_auto_positioned_boxes_among_siblings_only` | PR, quire (floating surfaces use the overlay host with a z-index) |
| Hoisted z-indexed boxes laid out after `resolve_layout` | `flush_styles_to_layout` runs before layout and records offsets from the previous layout, so a moved ancestor paints its raised child in the old place; hit test and `refresh_hover` read the stale lists | verified: shell-host/FINDINGS.md:140 (F69; `tests/hoisted.rs`); quire/FINDINGS.md:"Hit testing" | fork or PR, host (`rehoist` calls `flush_styles_to_layout` again), not filed |
| Hit test that clips a scroller's overflow | A point outside a scroll container over its hidden content latches the container; `overflow: hidden` clips paint, not hit | verified: shell-host/FINDINGS.md:651; quire/FINDINGS.md:"Hit testing" (`overflow: hidden` bullet) | host, quire (`pointer-events: none` on hidden lines), PR |
| Hit test and client rect that agree on transforms | Hit test and paint apply transforms; `get_client_bounding_rect` leaves them out | verified: quire/FINDINGS.md:720 | kit (`paint_rect::painted_rect`), quire (centre without a transform) |
| `get_client_rect` on a shared borrow | Borrows the document mutably, so a rect read can collide with the renderer | verified: quire/FINDINGS.md:31 | PR |
| Client rect that ignores the element's own scroll offset | `absolute_position` subtracts it, so a scrolled list's own rect moves with its content | verified: quire/FINDINGS.md:727 | host (`dom::scroll::area` adds it back), quire |
| Pointer capture | None; a drag leaving an element is noticed at the next move with no button down | verified: quire/FINDINGS.md:777 | quire (level control and swipe release; edit surface captures itself), PR |
| UI Events order for leave (innermost first) | Blitz sends leaves outermost first | verified: quire/FINDINGS.md:757 | quire (`onpointerback` independent of the order) |
| `click` only when press and release share a target | Sent to the release target even when the press began elsewhere | verified: quire/FINDINGS.md:763 | quire (menu picks once) |
| Middle and right buttons as ordinary pointer events | Right button is `contextmenu`, never `click`; middle is `mouseup` only; a press past a 2 px drag threshold dispatches no `click` | verified: quire/FINDINGS.md:765; shell-host/FINDINGS.md:727 | quire (`Button` listens to all three) |
| `mounted` rect reads to be valid inside `onmounted` | Mounted events are flushed before any layout, so the rect is 0 x 0 there | verified: quire/FINDINGS.md S9 | quire (`use_rect` reads a frame later) |
| Drag and drop and file-drop events | blitz-shell ignores winit's drag events and the document has none; paths are not carried by winit 0.31 | verified: quire/FINDINGS.md:645-660 | quire (`ds-native` reads them at the window hook) |

## Scrolling and wheel

| Want | Blitz today | Evidence | Route |
| --- | --- | --- | --- |
| Wheel events with phase and source (touchpad gesture end, momentum, wheel vs finger) | `BlitzWheelEvent` carries neither; `handle_wheel` scales lines by 20 px, targets the hover node (set only by a pointer move), and has no latching | verified: quire/FINDINGS.md:772; quire/design/11-BEHAVIOUR-scroll.md:44 (R21) | fork or PR, host (scroll engine owns every scroll), quire (`SwipeQuiet` 120 ms) |
| Touchpad flings and rubber-band on wheel input | `Fling` starts only from a touch pan (never sent); wheels never fling | verified: shell-host/FINDINGS.md:617-620; design/11 R21 | host (own physics), PR |
| Wheel delta with the web's sign | Forwarded unchanged; positive x moves content right | verified: quire/FINDINGS.md:772 | quire, host (negates) |
| Raw scroll setters that clamp and notify (`onscroll`, redraw, scrollbar wake) | `set_viewport_scroll` and `scroll_offset_mut()` do not clamp, do not redraw, emit no scroll event | verified: shell-host/FINDINGS.md:617-629; design/11 R21 | host (`use_scroll`, `ScrollCmd`), PR |
| A public way to inject scroll events for a scroller (a keyboard scroll action, page-up / down) | None: no keyboard scroll action, no public event injection that reaches scroll handlers | verified: shell-host/FINDINGS.md:629 | host (engine writes offsets each frame) |
| Blitz to report `prevent_default` on key listeners | Cannot; dioxus-native-dom stores listeners as attributes named for the event | verified: shell-host/FINDINGS.md:632 | host (a key listener on the focus path counts as consuming scroll keys) |
| `scroll-behavior: smooth` to cooperate with an external scroll engine | Blitz's 300 ms ease-in-out-cubic `ScrollTo` fights it | verified: shell-host/FINDINGS.md:620; quire/crates/ds/src/lint/blitz.rs:24 | quire (lint bans it), host (`ScrollCmd`) |
| `scroll_into_view` for nested scrollers | Scrolls the document viewport only | verified: quire/FINDINGS.md:729 | quire (`HostReveal`, `ds::nearest_scroll`) |
| Overlay scrollbars we can style (colour, thumb drag, macOS shape) | Chromium-like thumb (10 px, 500 ms + 200 ms fade), behind a `scrollbars` feature we keep off; `scrollbar-width: none` disables paint and hit test outright | verified: quire/FINDINGS.md:782; shell-host/FINDINGS.md:627 | host (paints its own thumb), quire (`scrollbar-width: none`) |
| Viewport that can carry `data-overscroll="band"` | `html` / `body` cannot get it from Dioxus | verified: shell-host/FINDINGS.md F63 "Open" | host, PR (dioxus-native-dom) |

## Threading, hosting and documents

| Want | Blitz today | Evidence | Route |
| --- | --- | --- | --- |
| Multi-threaded style resolution across many documents | `StyleThreading::Sequential` only, on the main thread (blitz #430) | verified: shell-host/crates/shell-host/src/dom/document/mod.rs:87 | host (all documents resolve on one thread), PR |
| `document::Style`, `Link` and `Title` processed by `DioxusDocument` | Not processed; a Dioxus `Document` shim is needed | verified: shell-host/crates/shell-host/src/dom/head.rs:1 | host (`HeadDocument` queues, host applies to `<head>` after each poll), PR (dioxus-native-dom) |
| One wgpu device shared by many surfaces | The stock `VelloWindowRenderer` owns a private `WGPUContext`; sharing needs `VelloScenePainter` around one renderer, and `ImageManager` / painter borrow caches | verified: shell-host/FINDINGS.md:11-15 (F1); plan "Findings" assumed research 2026-09-23 | host (one `vello_hybrid::Renderer` + `Resources`) |
| A second window's renderer to close without crashing the first | Closing it crashes the first window on NVIDIA / Wayland (`vkAcquireNextImageKHR` through a null pointer); untested on other drivers, not reported | verified: quire/FINDINGS.md:53 | quire (keeps closed renderers), PR once minimised |
| The loop not to exit when the last window of one application closes | blitz-shell exits when an application's last window closes | verified: quire/FINDINGS.md:639 | quire (relays second-window closes) |
| Custom widgets to paint without a feature flag | `blitz_paint::paint_scene` drives `Widget`s only with `blitz-paint/custom-widget`; without it they silently never paint | verified: shell-host/FINDINGS.md:159 | host (feature on) |
| Fonts shared across documents | One `FontContext` cloned into each `DocumentConfig.font_ctx` shares the source cache; nothing shares it by default | verified: quire/FINDINGS.md S11; blitz-kit `fonts` | kit (`SharedFonts`) |
| A `NetProvider` that serves `data:` and `file:` by default | Default is `DummyNetProvider`; `data:` masks and backgrounds never load (the element is fully masked); the image arrives one resolve late, so the first frame is blank | verified: quire/FINDINGS.md S7, S8; blitz-kit `net` | kit (`LocalNet`) |
| A `launch` path with pixel snapping and hover replay hooks | blitz-shell resolves and paints in one redraw with no hook between | verified: quire/FINDINGS.md:77 | quire (`ds-native` harness), host (pre-paint hook) |
| Blitz layout rounded to device pixels | Rounds to whole logical px only (`taffy::round_layout` on every resolve, undoing any earlier snap) | verified: shell-host/FINDINGS.md:130 (F43); quire/FINDINGS.md:668 | kit (`snap_layout` after the frame's one resolve), PR |

## Verified to work

- Runtime `<style>` text changes restyle the document, including custom-property overrides and an
  emptied sheet (`quire/crates/ds-native/tests/user_style_reload.rs`:
  `a_changed_style_text_restyles_the_probe`, `a_changed_custom_property_override_restyles_the_probe`).
- A `<style>` in the body applies (S1); custom properties, `var()`, inheritance and nested scopes
  cascade (S2, once selectors carry `*|`).
- `@keyframes` with `var()` inside, and `var()` durations, interpolate (S3); `transition` on
  `var()`-driven values runs (S4); swapping `animation-name` restarts an animation (S5).
- `color-mix(in srgb | in oklab, ...)` works, with `var()` inside (S14).
- `mask-image` (SVG and PNG) and tiled `background-image` paint with a `data:` `NetProvider`
  (S7, S8); mask layers composite with `add`; `min()` / `calc()` with percentages resolve in
  `mask-size`.
- A TTF registered in a shared `FontContext` is picked by `font-family` (S11); `font-variant-
  numeric` and `tnum` work.
- A futures timer wakes the document from render and from handlers (S10).
- `:where()` works; `pointer-events: none` is honoured and inherited; `caret-color: transparent`
  is honoured; `text-decoration` underline and line-through paint; `@media print` is honoured.
- `vh` / `vw` resolve against the viewport; `calc()` with `min()` and `var()` resolves in keyframe
  `scale()` and `translateY()`; five `box-shadow` layers paint.
- Offscreen GPU rendering through `wgpu_context::BufferRenderer` and `vello_hybrid::Renderer`
  works; vello_hybrid fits a 60 Hz frame for the 300-emoji grid (p95 5-7 ms).
- A scroll does not relayout: wheel plus style plus layout is 0.03 to 0.1 ms p50.
