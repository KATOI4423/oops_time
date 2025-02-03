######################
# Makefile for cargo #
######################

.PHONY: ALL
ALL: linux windows

.PHONY: linux
linux:
	cargo build --release

.PHONY: windows
windows:
	cargo build --release --target x86_64-pc-windows-gnu
