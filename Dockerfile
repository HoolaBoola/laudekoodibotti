FROM rust:latest as builder
RUN apt-get update && apt-get install -y clang libleptonica-dev libtesseract-dev
WORKDIR /app
COPY Cargo.lock Cargo.toml /app/
COPY src/ /app/src/
RUN cargo build --release
# Copy binary and its dependencies to dist folder
RUN mkdir -p dist/app && cp target/release/stickerreadbot dist/app && cp --parents $(ldd target/release/stickerreadbot | grep -P -o "/.+(?= \()") dist

FROM busybox:glibc
COPY --from=builder /app/dist/ /
COPY traineddata/ /app/traineddata/
WORKDIR /app
CMD ./stickerreadbot $TOKEN