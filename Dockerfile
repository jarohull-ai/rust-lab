FROM node:22-bookworm-slim AS ui-builder
WORKDIR /build/ui
COPY ui/package.json ui/package-lock.json ./
RUN npm ci
COPY ui/ ./
RUN npm run build

FROM rust:bookworm AS rust-builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
RUN cargo build --locked --release -p api

FROM rust:bookworm AS runtime
RUN useradd --create-home --shell /usr/sbin/nologin rustlab

WORKDIR /opt/rust-lab
COPY --from=rust-builder /build/target/release/api ./rust-lab
COPY --from=rust-builder /usr/local/cargo/registry /usr/local/cargo/registry
COPY --from=ui-builder /build/ui/dist ./ui/dist
COPY data/ ./data/
COPY examples/ ./examples/
RUN mkdir -p output target/exports \
    && chown -R rustlab:rustlab /opt/rust-lab/data /opt/rust-lab/output /opt/rust-lab/target \
    && chown -R rustlab:rustlab /usr/local/cargo/registry

USER rustlab
ENV RUST_LAB_HOST=0.0.0.0
ENV PORT=3030
EXPOSE 3030
ENTRYPOINT ["./rust-lab"]
