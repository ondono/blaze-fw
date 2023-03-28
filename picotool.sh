#!/bin/sh
set -eu

# To use as a cargo runner, set:
#
# [target.YOUR_TARGET_NAME]
# runner = "./picotool.sh" # or wherever you put this script
#
# in .cargo/config.toml
# YOUR_TARGET_NAME is probably thumbv6m-none-eabi for the pico

TARGET="$(grep "^target = " ./.cargo/config.toml | head -n 1 | awk -F'\"' '{print $2}')"
PROJECT="$(grep "^name = " ./Cargo.toml | head -n 1 | awk -F'\"' '{print $2}')"
ELF_FILE="./target/$TARGET/release/$PROJECT"

cp -v "$ELF_FILE" "$ELF_FILE.elf"
picotool load "$ELF_FILE.elf" -f
picotool reboot
