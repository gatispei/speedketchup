FROM docker.io/library/debian:trixie
RUN apt-get update && apt-get install -y --no-install-recommends git ca-certificates cmake clang make zlib1g-dev libicu-dev python3 && rm -rf /var/lib/apt/lists/*
RUN git clone --depth 1 https://github.com/microsoft/msix-packaging /msix
WORKDIR /msix
# the system ICU headers want C++17
RUN sed -i 's/set(CMAKE_CXX_STANDARD 14)/set(CMAKE_CXX_STANDARD 17)/' CMakeLists.txt lib/xerces/CMakeLists.txt
RUN ./makelinux.sh --pack --skip-samples --skip-tests -b MinSizeRel > /build.log 2>&1 || (tail -40 /build.log; exit 1)
RUN find /msix -name makemsix -type f; find /msix -name 'libmsix*.so*'
