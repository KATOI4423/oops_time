######################
# Makefile for cargo #
######################

# Directory
TOP_DIR := $(shell pwd)

# Source Directory
SRC_DIR := ${TOP_DIR}/src
TAURI_DIR := ${TOP_DIR}/src-tauri

# Build Directory
BUILD_DIR := ${TOP_DIR}/build
BUILD_LINUX_DIR := ${BUILD_DIR}/release
BUILD_WINDOWS_DIR := ${BUILD_DIR}/x86_64-pc-windows-gnu/release

$(shell mkdir -p ${BUILD_DIR})

.PHONY: all
all: linux windows

.PHONY: clean
clean: linux-clean windows-clean

.PHONY: dist-clean
dist-clean:
	rm -rf ${BUILD_DIR}/*

.PHONY: linux
linux:
	@cargo tauri build -- --target-dir ${BUILD_DIR}

.PHONY: linux-clean
linux-clean:
	rm -rf ${BUILD_LINUX_DIR}

.PHONY: windows
windows:
	@. ./setenv_windows.sh \
	&& cargo tauri build --target x86_64-pc-windows-gnu -- --target-dir ${BUILD_DIR}

.PHONY: windows-clean
windows-clean:
	rm -rf ${BUILD_WINDOWS_DIR}
