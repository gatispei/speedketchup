FROM debian:bookworm

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
	ca-certificates curl file make xz-utils \
	gcc libc6-dev \
	gcc-x86-64-linux-gnu \
	gcc-i686-linux-gnu \
	gcc-aarch64-linux-gnu \
	gcc-arm-linux-gnueabi \
	gcc-mips-linux-gnu \
	gcc-mipsel-linux-gnu \
	gcc-mingw-w64-x86-64 \
	qemu-user-static \
	&& rm -rf /var/lib/apt/lists/*

ARG UPX_VERSION=5.2.1
RUN set -eux; \
	case "$(dpkg --print-architecture)" in \
		arm64) upx_arch=arm64 ;; \
		amd64) upx_arch=amd64 ;; \
		*) echo "unsupported build host architecture" >&2; exit 1 ;; \
	esac; \
	curl -fsSL "https://github.com/upx/upx/releases/download/v${UPX_VERSION}/upx-${UPX_VERSION}-${upx_arch}_linux.tar.xz" \
		| tar -xJ -C /usr/local/bin --strip-components=1 --wildcards '*/upx'; \
	upx --version | head -1

ARG MUSL_VERSION=1.2.6
RUN set -eux; \
	curl -fsSL "https://musl.libc.org/releases/musl-${MUSL_VERSION}.tar.gz" | tar -xz -C /tmp; \
	for arch in mips mipsel; do \
		mkdir "/tmp/musl-build-$arch"; cd "/tmp/musl-build-$arch"; \
		CC="$arch-linux-gnu-gcc" AR="$arch-linux-gnu-ar" RANLIB="$arch-linux-gnu-ranlib" \
		CFLAGS="-msoft-float" \
			"/tmp/musl-${MUSL_VERSION}/configure" \
				--target="$arch-linux-musl" \
				--prefix="/opt/musl/$arch" \
				--disable-shared >/dev/null; \
		make -j"$(nproc)" >/dev/null; \
		make install >/dev/null; \
		"$arch-linux-gnu-ar" rcs "/opt/musl/$arch/lib/libunwind.a"; \
		printf '#!/bin/sh\nexec %s-linux-gnu-gcc -msoft-float -B/opt/musl/%s/lib -L/opt/musl/%s/lib "$@"\n' \
			"$arch" "$arch" "$arch" > "/usr/local/bin/$arch-linux-musl-gcc"; \
		chmod +x "/usr/local/bin/$arch-linux-musl-gcc"; \
	done; \
	cd /; rm -rf "/tmp/musl-${MUSL_VERSION}" /tmp/musl-build-mips /tmp/musl-build-mipsel

ENV RUSTUP_HOME=/usr/local/rustup \
	CARGO_HOME=/usr/local/cargo \
	PATH=/usr/local/cargo/bin:$PATH

ARG RUST_NIGHTLY=nightly
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y \
	--default-toolchain "${RUST_NIGHTLY}" \
	--profile minimal \
	--component rust-src \
	&& rustup target add \
		x86_64-unknown-linux-musl \
		i686-unknown-linux-musl \
		aarch64-unknown-linux-musl \
		arm-unknown-linux-musleabi \
		x86_64-pc-windows-gnu \
	&& rustc --version

WORKDIR /src
