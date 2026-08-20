#!/usr/bin/env bash
set -euo pipefail
# omatunes-TEST isolated launcher — never touches real ~/.config/omatunes or ~/.local/share
# Mirrors live launch in ~/.config/hypr/bindings.lua:
#   uwsm-app -- env VK_DRIVER_FILES=/usr/share/vulkan/icd.d/nvidia_icd.json __NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia /home/davepople/.local/bin/omatunes
# plus WGPU fallback (src/main.rs) so black-screen on dual-GPU (Nvidia+AMD) is avoided.
TEST_ROOT="${XDG_RUNTIME_DIR:-/tmp}/omatunes-test-$$"
export HOME="$TEST_ROOT/home"
export XDG_CONFIG_HOME="$TEST_ROOT/config"
export XDG_DATA_HOME="$TEST_ROOT/data"
export XDG_CACHE_HOME="$TEST_ROOT/cache"
# GPU prime offload — match live Hyprland env so wgpu picks Nvidia not iGPU
export VK_DRIVER_FILES=/usr/share/vulkan/icd.d/nvidia_icd.json
export __NV_PRIME_RENDER_OFFLOAD=1
export __GLX_VENDOR_LIBRARY_NAME=nvidia
# wgpu backend fallback (also set in src/main.rs if unset, but export here for child processes)
if [ -z "${WGPU_BACKEND:-}" ]; then
  export WGPU_BACKEND="gl,vulkan"
fi
mkdir -p "$HOME/.config/omatunes" "$XDG_CONFIG_HOME/omatunes" "$XDG_DATA_HOME" "$XDG_CACHE_HOME"
echo "omatunes-TEST isolated dirs:"
echo "  HOME=$HOME"
echo "  XDG_CONFIG_HOME=$XDG_CONFIG_HOME"
echo "  XDG_DATA_HOME=$XDG_DATA_HOME"
echo "  XDG_CACHE_HOME=$XDG_CACHE_HOME"
echo "  VK_DRIVER_FILES=$VK_DRIVER_FILES __NV_PRIME_RENDER_OFFLOAD=$__NV_PRIME_RENDER_OFFLOAD __GLX_VENDOR_LIBRARY_NAME=$__GLX_VENDOR_LIBRARY_NAME WGPU_BACKEND=$WGPU_BACKEND"
echo "  binary=/home/davepople/Projects/omaTUNES/target/release/omatunes-TEST"
echo "  real config untouched: ~/.config/omatunes not read (HOME overridden)"
exec /home/davepople/Projects/omaTUNES/target/release/omatunes-TEST "$@"
