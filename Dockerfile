FROM rust:latest as builder
RUN apt-get update && apt-get install -y clang libleptonica-dev libtesseract-dev
WORKDIR /app
# Build a dummy project with dependencies first and sources later. This way the dependencies are
# already built and cached every time the sources change.
RUN USER=root cargo init
COPY Cargo.lock Cargo.toml /app/
RUN cargo build --release
RUN rm -r src && rm -r target/release/.fingerprint/stickerreadbot*
COPY src/ /app/src/
RUN cargo build --release
# Copy binary and its dependencies to dist folder
RUN mkdir -p dist/app && cp target/release/stickerreadbot dist/app && cp --parents $(ldd target/release/stickerreadbot | grep -P -o "/.+(?= \()") dist

FROM busybox:glibc
COPY --from=builder /app/dist/ /
COPY traineddata/ /app/traineddata/
WORKDIR /app
CMD ./stickerreadbot $TOKEN