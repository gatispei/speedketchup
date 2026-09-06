NIGHTLY   := nightly-2026-09-06
CONTAINER ?= container
IMAGE     := speedketchup-build

VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
SOURCES := Makefile Cargo.toml $(wildcard Cargo.lock src/*.rs asset/*)

CARGO   := cargo +$(NIGHTLY) build -Z build-std=std,panic_abort --release
RFLAGS  := -Zlocation-detail=none -Zunstable-options -Cpanic=immediate-abort

LINUX   := x64 i686 aarch64 arm mips mipsel
WINDOWS := x64.exe
MACOS   := macos-aarch64 macos-x64

x64_TARGET     := x86_64-unknown-linux-musl
i686_TARGET    := i686-unknown-linux-musl
aarch64_TARGET := aarch64-unknown-linux-musl
arm_TARGET     := arm-unknown-linux-musleabi
mips_TARGET    := mips-unknown-linux-musl
mipsel_TARGET  := mipsel-unknown-linux-musl
x64.exe_TARGET := x86_64-pc-windows-gnu

x64_STRIP     := x86_64-linux-gnu-strip
i686_STRIP    := i686-linux-gnu-strip
aarch64_STRIP := aarch64-linux-gnu-strip
arm_STRIP     := arm-linux-gnueabi-strip
mips_STRIP    := mips-linux-gnu-strip
mipsel_STRIP  := mipsel-linux-gnu-strip
x64.exe_STRIP := x86_64-w64-mingw32-strip

x64_QEMU     := qemu-x86_64-static
i686_QEMU    := qemu-i386-static
aarch64_QEMU := qemu-aarch64-static
arm_QEMU     := qemu-arm-static
mips_QEMU    := qemu-mips-static
mipsel_QEMU  := qemu-mipsel-static

x64.exe_BIN := speedketchup.exe

arm_RFLAGS    := -Clink-arg=-lgcc
mips_RFLAGS   := -Ctarget-feature=+crt-static -Clink-self-contained=no -Alinker_messages
mipsel_RFLAGS := $(mips_RFLAGS)

macos-aarch64_TARGET := aarch64-apple-darwin
macos-x64_TARGET     := x86_64-apple-darwin
macos-aarch64_STRIP  := strip
macos-x64_STRIP      := strip

linux_BINS   := $(addprefix bin/speedketchup-,$(LINUX) $(WINDOWS))
linux_PACKED := $(addprefix bin/speedketchup-,$(addsuffix -upx,$(LINUX)) x64-upx.exe)
macos_BINS   := $(addprefix bin/speedketchup-,$(MACOS))

.PHONY: build release macos linux image clean smoke
.DEFAULT_GOAL := build

build:
	cargo build --release

release: macos linux

macos: bin/speedketchup-macos

bin:
	mkdir -p $@

bin/speedketchup-%: $(SOURCES) | bin
	RUSTFLAGS="$(RFLAGS) $($*_RFLAGS)" $(CARGO) --target $($*_TARGET)
	cp $(CARGO_TARGET_DIR)/$($*_TARGET)/release/$(or $($*_BIN),speedketchup) $@
	$($*_STRIP) $@

bin/speedketchup-macos: $(macos_BINS)
	lipo -create -output $@ $^
	@printf '%-22s %s\n' macos "$$(./$@ --version 2>&1 | head -1)"

bin/speedketchup-%-upx: bin/speedketchup-%
	@rm -f $@
	upx -qq --ultra-brute -o $@ $<

bin/speedketchup-%-upx.exe: bin/speedketchup-%.exe
	@rm -f $@
	upx -qq --ultra-brute -o $@ $<

image:
	$(CONTAINER) build -t $(IMAGE) -f build/Dockerfile \
		--build-arg RUST_NIGHTLY=$(NIGHTLY) build

clean:
	rm -rf target target-linux bin

ifeq ($(SPEEDKETCHUP_IN_CONTAINER),1)

export CARGO_TARGET_DIR := target-linux

export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER := x86_64-linux-gnu-gcc
export CARGO_TARGET_I686_UNKNOWN_LINUX_MUSL_LINKER   := i686-linux-gnu-gcc
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER     := x86_64-w64-mingw32-gcc

linux: smoke

define smoketest
got=$$(QEMU_PAGESIZE=$(4) $(2) $(3) --version 2>&1 | head -1); \
if [ "$$got" = "$(VERSION)" ]; then printf '%-22s ok\n' "$(1)"; \
else printf '%-22s FAILED: %s\n' "$(1)" "$$got"; fail=1; fi;
endef

smoke: $(linux_BINS) $(linux_PACKED)
	@fail=0; \
	$(foreach p,$(LINUX),\
		$(call smoketest,$(p),$($(p)_QEMU),bin/speedketchup-$(p),4096) \
		$(call smoketest,$(p)-upx,$($(p)_QEMU),bin/speedketchup-$(p)-upx,4096)) \
	$(call smoketest,aarch64-upx 64k pages,qemu-aarch64-static,bin/speedketchup-aarch64-upx,65536) \
	printf '%-22s skipped (no emulator)\n' x64.exe; \
	[ $$fail -eq 0 ] || { echo "smoke test failed" >&2; exit 1; }

else

CARGO_TARGET_DIR := target
NCPU := $(shell sysctl -n hw.ncpu 2>/dev/null || nproc)

in_container = $(CONTAINER) run --rm -c $(NCPU) -m 8g \
	-e SPEEDKETCHUP_IN_CONTAINER=1 \
	-v "$(CURDIR):/src" -w /src \
	$(IMAGE) make

linux: image
	$(in_container) linux

$(linux_BINS) $(linux_PACKED): image
	$(in_container) $@

macos: | toolchain
.PHONY: toolchain
toolchain:
	@rustup toolchain list | grep -q '^$(NIGHTLY)' || \
		rustup toolchain install $(NIGHTLY) --profile minimal
	@rustup component list --toolchain $(NIGHTLY) --installed | grep -q '^rust-src$$' || \
		rustup component add rust-src --toolchain $(NIGHTLY)
	@rustup target list --toolchain $(NIGHTLY) --installed | grep -q '^$(macos-x64_TARGET)$$' || \
		rustup target add --toolchain $(NIGHTLY) $(macos-aarch64_TARGET) $(macos-x64_TARGET)

endif
