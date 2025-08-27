IMAGE_NAME := speedketchup
DOCKERHUB_REPO := antonsmt/speedketchup

.PHONY: all clean x86 arm arm64 container-x86 container-arm container-arm64 push-arm push-arm64 push-amd64 push-manifest push-all

all: x86 arm arm64

x86:
	cargo build --release --target x86_64-unknown-linux-musl
	upx --best target/x86_64-unknown-linux-musl/release/speedketchup

arm:
	cargo build --release --target arm-unknown-linux-musleabi
	upx --best target/arm-unknown-linux-musleabi/release/speedketchup

arm64:
	cargo build --release --target aarch64-unknown-linux-musl
	upx --best target/aarch64-unknown-linux-musl/release/speedketchup

# Container build targets using locally built binaries
container-x86: x86
	@echo "Building x86_64 container..."
	podman build --platform linux/amd64 -t $(IMAGE_NAME):x86 -t $(DOCKERHUB_REPO):amd64 --build-arg ARCH=x86_64 .
	podman save $(IMAGE_NAME):x86 -o speedketchup-x86.tar
	@echo "x86_64 container saved as speedketchup-x86.tar"

container-arm: arm
	@echo "Building ARM container..."
	podman build --platform linux/arm/v7 -t $(IMAGE_NAME):arm -t $(DOCKERHUB_REPO):arm --build-arg ARCH=arm .
	podman save $(IMAGE_NAME):arm -o speedketchup-arm.tar
	@echo "ARM container saved as speedketchup-arm.tar"

container-arm64: arm64
	@echo "Building ARM64 container..."
	podman build --platform linux/arm64 -t $(IMAGE_NAME):arm64 -t $(DOCKERHUB_REPO):arm64 --build-arg ARCH=aarch64 .
	podman save $(IMAGE_NAME):arm64 -o speedketchup-arm64.tar
	@echo "ARM64 container saved as speedketchup-arm64.tar"

clean:
	cargo clean
	rm -f speedketchup-*.tar
	podman rmi $(IMAGE_NAME):arm $(IMAGE_NAME):arm64 $(IMAGE_NAME):x86 2>/dev/null || true
	@echo "Cleaned up temporary files and container images"