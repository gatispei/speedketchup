NIGHTLY   := nightly-2026-09-06
CONTAINER ?= container
BUILD_IMAGE := speedketchup-build

IMAGE       ?= docker.io/gatispei/speedketchup
HUB_REPO    ?= gatispei/speedketchup
HUB_USER    ?= gatispei

-include .env
export DOCKERHUB_TOKEN
IMAGE_BUILD ?= $(CONTAINER) build
PLATFORMS   := linux/amd64,linux/386,linux/arm64,linux/arm/v6
IMAGE_ARCHS := amd64=x64 386=i686 arm64=aarch64 arm=arm

VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
SOURCES := Makefile Cargo.toml build.rs $(wildcard Cargo.lock src/*.rs asset/* windows/*.ico)

# build.rs minifies the scripts in asset/index.html, release builds require it
ESBUILD_VERSION := 0.25.10
ESBUILD ?= npx --yes esbuild@$(ESBUILD_VERSION)

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

.PHONY: build release macos app linux image qpkg build-image push overview clean smoke mas mas-upload msix msix-image
.DEFAULT_GOAL := build

build:
	cargo build --release

release: macos app linux

macos: bin/speedketchup-macos

bin:
	mkdir -p $@

bin/speedketchup-%: $(SOURCES) | bin
	SPEEDKETCHUP_MINIFY=1 ESBUILD="$(ESBUILD)" \
	RUSTFLAGS="$(RFLAGS) $($*_RFLAGS)" $(CARGO) --target $($*_TARGET)
	cp $(CARGO_TARGET_DIR)/$($*_TARGET)/release/$(or $($*_BIN),speedketchup) $@
	$($*_STRIP) $@

bin/speedketchup-macos: $(macos_BINS)
	lipo -create -output $@ $^
	@printf '%-22s %s\n' macos "$$(./$@ --version 2>&1 | head -1)"

# SpeedKetchup.app, the macos binary is a menu bar icon when it runs from a
# bundle. The zip is signed ad hoc, apple silicon runs nothing unsigned, or
# with DEVELOPER_ID and notarized with the notarytool NOTARY_PROFILE keychain
# profile. BUILD is CFBundleVersion, every App Store upload needs a higher one
DEVELOPER_ID   ?=
NOTARY_PROFILE ?= speedketchup
BUILD          ?= $(VERSION)
APP_FILES      := bin/speedketchup-macos macos/Info.plist macos/SpeedKetchup.icns macos/SpeedKetchup.entitlements

# the Mac App Store package: the app sandboxed, signed with the Apple
# Distribution certificate and the provisioning profile of TEAM_ID, in a pkg
# signed with the Mac Installer Distribution certificate. mas-upload sends it
# to App Store Connect with the API key ASC_KEY_ID of issuer ASC_ISSUER, its
# AuthKey_<ASC_KEY_ID>.p8 in ~/.appstoreconnect/private_keys
TEAM_ID        ?=
MAS_APP_ID     ?= Apple Distribution
MAS_PKG_ID     ?= 3rd Party Mac Developer Installer
MAS_PROFILE    ?= macos/SpeedKetchup.provisionprofile
ASC_KEY_ID     ?=
ASC_ISSUER     ?=

# $(1) the bundle directory
define make_bundle
	rm -rf $(1)/SpeedKetchup.app
	mkdir -p $(1)/SpeedKetchup.app/Contents/MacOS $(1)/SpeedKetchup.app/Contents/Resources
	cp bin/speedketchup-macos $(1)/SpeedKetchup.app/Contents/MacOS/speedketchup
	cp macos/SpeedKetchup.icns $(1)/SpeedKetchup.app/Contents/Resources/
	sed 's/@VERSION@/$(VERSION)/g; s/@BUILD@/$(BUILD)/g' macos/Info.plist > $(1)/SpeedKetchup.app/Contents/Info.plist
endef

app: bin/SpeedKetchup-macos.zip

bin/SpeedKetchup-macos.zip: $(APP_FILES)
	rm -f $@
	$(call make_bundle,bin)
ifeq ($(DEVELOPER_ID),)
	codesign --force --sign - bin/SpeedKetchup.app
	cd bin && ditto -c -k --keepParent SpeedKetchup.app SpeedKetchup-macos.zip
else
	codesign --force --options runtime --timestamp --sign "$(DEVELOPER_ID)" bin/SpeedKetchup.app
	cd bin && ditto -c -k --keepParent SpeedKetchup.app SpeedKetchup-macos.zip
	xcrun notarytool submit $@ --keychain-profile "$(NOTARY_PROFILE)" --wait
	xcrun stapler staple bin/SpeedKetchup.app
	rm $@ && cd bin && ditto -c -k --keepParent SpeedKetchup.app SpeedKetchup-macos.zip
endif

mas: bin/SpeedKetchup.pkg

bin/SpeedKetchup.pkg: $(APP_FILES) $(MAS_PROFILE)
	@test -n "$(TEAM_ID)" || { echo "TEAM_ID is not set"; exit 1; }
	rm -f $@
	$(call make_bundle,bin/mas)
	cp $(MAS_PROFILE) bin/mas/SpeedKetchup.app/Contents/embedded.provisionprofile
	cp macos/SpeedKetchup.entitlements bin/mas/entitlements.plist
	plutil -insert 'com\.apple\.application-identifier' -string $(TEAM_ID).com.speedketchup.SpeedKetchup bin/mas/entitlements.plist
	plutil -insert 'com\.apple\.developer\.team-identifier' -string $(TEAM_ID) bin/mas/entitlements.plist
	codesign --force --options runtime --timestamp --entitlements bin/mas/entitlements.plist --sign "$(MAS_APP_ID)" bin/mas/SpeedKetchup.app
	productbuild --component bin/mas/SpeedKetchup.app /Applications --sign "$(MAS_PKG_ID)" $@

mas-upload: bin/SpeedKetchup.pkg
	xcrun altool --validate-app -f $< -t macos --apiKey $(ASC_KEY_ID) --apiIssuer $(ASC_ISSUER)
	xcrun altool --upload-app -f $< -t macos --apiKey $(ASC_KEY_ID) --apiIssuer $(ASC_ISSUER)

# Microsoft Store package, the identity comes from .env
MSIX_IDENTITY       ?= SpeedKetchup
MSIX_PUBLISHER      ?= CN=SpeedKetchup
MSIX_PUBLISHER_NAME ?= SpeedKetchup
MSIX_IMAGE          := speedketchup-msix

msix: bin/SpeedKetchup.msix

msix-image:
	$(IMAGE_BUILD) -t $(MSIX_IMAGE) -f build/msix.Dockerfile build

bin/SpeedKetchup.msix: bin/speedketchup-x64.exe windows/AppxManifest.xml $(wildcard windows/Assets/*.png) | msix-image
	rm -rf bin/msix $@
	mkdir -p bin/msix/Assets
	cp bin/speedketchup-x64.exe bin/msix/SpeedKetchup.exe
	cp windows/Assets/*.png bin/msix/Assets/
	sed 's|@IDENTITY@|$(MSIX_IDENTITY)|; s|@PUBLISHER@|$(MSIX_PUBLISHER)|; s|@PUBLISHER_NAME@|$(MSIX_PUBLISHER_NAME)|; s|@VERSION@|$(VERSION)|' \
		windows/AppxManifest.xml > bin/msix/AppxManifest.xml
	$(CONTAINER) run --rm -v "$(CURDIR):/src" -w /src $(MSIX_IMAGE) /msix/.vs/bin/makemsix pack -d bin/msix -p $@

bin/speedketchup-%-upx: bin/speedketchup-%
	@rm -f $@
	upx -qq --ultra-brute -o $@ $<

bin/speedketchup-%-upx.exe: bin/speedketchup-%.exe
	@rm -f $@
	upx -qq --ultra-brute -o $@ $<

build-image:
	$(CONTAINER) build -t $(BUILD_IMAGE) -f build/Dockerfile \
		--build-arg RUST_NIGHTLY=$(NIGHTLY) \
		--build-arg ESBUILD_VERSION=$(ESBUILD_VERSION) build

image: $(addprefix bin/speedketchup-,x64 i686 aarch64 arm)
	rm -rf bin/docker
	mkdir -p bin/docker
	@for m in $(IMAGE_ARCHS); do \
		mkdir -p bin/docker/$${m%%=*}/data; \
		cp bin/speedketchup-$${m#*=} bin/docker/$${m%%=*}/speedketchup; \
	done
	$(IMAGE_BUILD) --platform $(PLATFORMS) -t $(IMAGE):$(VERSION) -t $(IMAGE):latest .

# QNAP packages for x86_64, arm_64 and arm-x41, built with QDK in a debian container
qpkg: $(addprefix bin/speedketchup-,x64 aarch64 arm)
	$(CONTAINER) run --rm -v "$(CURDIR):/src" -w /src docker.io/library/debian:trixie sh qnap/build.sh $(VERSION)

push: image
	@case "$(IMAGE)" in \
	*/*) ;; \
	*) echo "IMAGE needs a namespace, eg make push IMAGE=docker.io/you/speedketchup" >&2; exit 1 ;; \
	esac
	$(CONTAINER) image push $(IMAGE):$(VERSION)
	$(CONTAINER) image push $(IMAGE):latest
	@$(MAKE) --no-print-directory overview

overview: DOCKERHUB.md
	@if [ -z "$$DOCKERHUB_TOKEN" ]; then \
		echo "no DOCKERHUB_TOKEN in .env, docker hub overview not updated" >&2; \
	else \
		jwt=$$(curl -sf -H "Content-Type: application/json" \
			-d "{\"username\": \"$(HUB_USER)\", \"password\": \"$$DOCKERHUB_TOKEN\"}" \
			https://hub.docker.com/v2/users/login/ \
			| python3 -c "import json,sys; print(json.load(sys.stdin)['token'])") && \
		code=$$(python3 -c "import json; print(json.dumps({'full_description': open('DOCKERHUB.md').read()}))" \
			| curl -s -X PATCH -d @- -o /dev/null -w '%{http_code}' \
				-H "Content-Type: application/json" -H "Authorization: JWT $$jwt" \
				https://hub.docker.com/v2/repositories/$(HUB_REPO)/); \
		case "$$code" in \
		2*) echo "docker hub overview updated" ;; \
		403) echo "docker hub refused the overview, DOCKERHUB_TOKEN has scope repo:write and needs read, write and delete" >&2; exit 1 ;; \
		*) echo "docker hub overview failed, http $$code" >&2; exit 1 ;; \
		esac; \
	fi

clean:
	rm -rf target target-linux bin

ifeq ($(SPEEDKETCHUP_IN_CONTAINER),1)

export CARGO_TARGET_DIR := target-linux
ESBUILD := esbuild

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
	$(BUILD_IMAGE) make

linux: build-image
	$(in_container) linux

$(linux_BINS) $(linux_PACKED): build-image
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
