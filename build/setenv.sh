#!/bin/bash

# Directory
export TOP_DIR="$(pwd)/.."

# Source Directory
export SRC_DIR="${TOP_DIR}/src"
export TAURI_DIR="${TOP_DIR}/src-tauri"

# Build Directory
export BUILD_DIR="${TOP_DIR}/build"
export BUILD_LINUX_DIR="${BUILD_DIR}/release"
export BUILD_WINDOWS_DIR="${BUILD_DIR}/x86_64-pc-windows-gnu/release"
export BUILD_REG_DIR="${BUILD_WINDOWS_DIR}/reg"

# Setting
export APPNAME=$(awk '
	/^\[package\]/ {found=1; next}
	/^\[/ {if (found) exit}
	found && /^name/ {
		match($0, /name = "([^"]+)"/, arr)
		print arr[1]
		exit
	}
' "${TAURI_DIR}/Cargo.toml")

export VERSION=$(awk '
	/^\[package\]/ {found=1; next}
	/^\[/ {if (found) exit}
	found && /^version/ {
		match($0, /version = "([^"]+)"/, arr)
		print arr[1]
		exit
	}
' "${TAURI_DIR}/Cargo.toml")

export AUMID="com.oopstime.app"

export PS1="${APPNAME}:${PS1}"
