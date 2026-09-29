#!/usr/bin/env bash
# blitz-kit stands below every consumer: no design system, no Dioxus, no Wayland, no shell.
#
# `cargo tree` lists every normal and build dependency of the kit with all features on; any
# package whose name starts with a forbidden prefix is a leak. Prefixes cover a family (dioxus*,
# wayland-*, ds-*) without spelling out its members. wgpu is pruned from the walk: its own
# backends (wgpu-hal's EGL path) link `wayland-sys` for their own windows, which is the GPU
# driver's business and no Wayland use by the kit; `wgpu` itself is a listed dependency.
set -uo pipefail
cd "$(dirname "$0")/.."

FORBIDDEN='^(ds|dioxus|wayland|smithay|cosmic|quire|sill|shell-host|palmrest)(-|_|$)'
fail=0

if ! cargo tree -p blitz-kit --depth 0 >/dev/null 2>&1; then
  echo "ERROR: cargo tree cannot resolve blitz-kit; the boundary was not checked"
  exit 1
fi

leaks=$(cargo tree -p blitz-kit --all-features -e normal,build --prefix none --no-dedupe --prune wgpu \
  | awk '{print $1}' | sort -u | grep -E "$FORBIDDEN" || true)
if [ -n "$leaks" ]; then
  echo "LEAK: blitz-kit depends on:"
  echo "$leaks"
  fail=1
else
  echo "boundary holds: blitz-kit reaches no ds*, dioxus*, wayland*, smithay*, cosmic*, quire, sill, shell-host or palmrest crate"
fi

exit "$fail"
