FROM scratch
ARG ARCH
COPY target/${ARCH}-unknown-linux-musl*/release/speedketchup /speedketchup
EXPOSE 8080
ENTRYPOINT ["/speedketchup"]