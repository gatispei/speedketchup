#!/bin/sh
# Builds the QNAP packages into bin/ from the linux binaries already in bin/,
# runs in a debian container, see `make qpkg`
#   build.sh VERSION

set -eu

VERSION=$1
QDK_VERSION=2.5.3
QDK_SHA256_amd64=17b3841b7d4590a4ee025844ba583304b5e3c497d9fa8934d5175131d3908022
QDK_SHA256_arm64=4b00c009cb48c0ffa7e4b7b00c5a6a1982a0955d663c0c6ec57020353e68eeb9

arch=$(dpkg --print-architecture)
eval sha256=\$QDK_SHA256_$arch
apt-get update -qq
apt-get install -y -qq --no-install-recommends ca-certificates curl > /dev/null
curl -fsSL -o /tmp/qdk.deb "https://github.com/qnap-dev/QDK/releases/download/v$QDK_VERSION/qdk_${QDK_VERSION}_$arch.deb"
echo "$sha256  /tmp/qdk.deb" | sha256sum -c -
apt-get install -y -qq /tmp/qdk.deb > /dev/null

# qbuild works in place, build in a copy so the source tree stays clean
rm -rf /tmp/qpkg
cp -R qnap /tmp/qpkg
sed -i "s/^QPKG_VER=\"\"/QPKG_VER=\"$VERSION\"/" /tmp/qpkg/qpkg.cfg
for m in x86_64=x64 arm_64=aarch64 arm-x41=arm; do
	mkdir -p /tmp/qpkg/${m%%=*}
	install -m 755 bin/speedketchup-${m#*=} /tmp/qpkg/${m%%=*}/speedketchup
done
chmod 755 /tmp/qpkg/shared/SpeedKetchup.sh
rm /tmp/qpkg/build.sh

cd /tmp/qpkg
qbuild
cd - > /dev/null
cp /tmp/qpkg/build/*.qpkg bin/
