# syntax=docker/dockerfile:1.5

FROM rust:1 as base

LABEL maintainer="adorsys Cameroon"

ENV CARGO_TERM_COLOR=always
ENV OPENSSL_STATIC=1

WORKDIR /app

FROM base as builder

ARG TARGETARCH

# Install toolchain and dependencies for static musl builds with vendored OpenSSL
RUN \
  --mount=type=cache,target=/var/cache/apt,sharing=locked \
  --mount=type=cache,target=/var/lib/apt,sharing=locked \
  apt-get update && \
  apt-get install -y --no-install-recommends \
    musl-tools \
    build-essential \
    pkg-config \
    perl \
  && rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

RUN \
  # Mount workspace files and only the necessary crates
  --mount=type=bind,source=./static,target=/app/static \
  --mount=type=bind,source=./Cargo.toml,target=/app/Cargo.toml \
  --mount=type=bind,source=./Cargo.lock,target=/app/Cargo.lock \
  --mount=type=bind,source=./crates/vym-fyi-server-crud/Cargo.toml,target=/app/crates/vym-fyi-server-crud/Cargo.toml \
  --mount=type=bind,source=./crates/vym-fyi-server-crud/migrations,target=/app/crates/vym-fyi-server-crud/migrations \
  --mount=type=bind,source=./crates/vym-fyi-server-crud/src,target=/app/crates/vym-fyi-server-crud/src \
  --mount=type=bind,source=./crates/vym-fyi-model/Cargo.toml,target=/app/crates/vym-fyi-model/Cargo.toml \
  --mount=type=bind,source=./crates/vym-fyi-model/src,target=/app/crates/vym-fyi-model/src \
  --mount=type=bind,source=./crates/vym-fyi-node/Cargo.toml,target=/app/crates/vym-fyi-node/Cargo.toml \
  --mount=type=bind,source=./crates/vym-fyi-node/src,target=/app/crates/vym-fyi-node/src \
  --mount=type=bind,source=./crates/vym-fyi-client/Cargo.toml,target=/app/crates/vym-fyi-client/Cargo.toml \
  --mount=type=bind,source=./crates/vym-fyi-client/src,target=/app/crates/vym-fyi-client/src \
  --mount=type=bind,source=./crates/vym-fyi-server-redirect/Cargo.toml,target=/app/crates/vym-fyi-server-redirect/Cargo.toml \
  --mount=type=bind,source=./crates/vym-fyi-server-redirect/src,target=/app/crates/vym-fyi-server-redirect/src \
  --mount=type=bind,source=./crates/vym-fyi-healthcheck/Cargo.toml,target=/app/crates/vym-fyi-healthcheck/Cargo.toml \
  --mount=type=bind,source=./crates/vym-fyi-healthcheck/src,target=/app/crates/vym-fyi-healthcheck/src \
  --mount=type=cache,target=/app/target \
  --mount=type=cache,target=/usr/local/cargo/registry/cache \
  --mount=type=cache,target=/usr/local/cargo/registry/index \
  --mount=type=cache,target=/usr/local/cargo/git/db \
  case "$TARGETARCH" in \
    "amd64") \
      export RUST_TARGET=x86_64-unknown-linux-musl; \
      export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc; \
      ;; \
    "arm64") \
      export RUST_TARGET=aarch64-unknown-linux-musl; \
      export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc; \
      ;; \
    *) \
      echo "Unsupported TARGETARCH: $TARGETARCH"; \
      exit 1; \
      ;; \
  esac; \
  cargo build --profile prod --locked \
    --target "${RUST_TARGET}" \
    -p vym-fyi-server-crud \
    -p vym-fyi-server-redirect \
    -p vym-fyi-healthcheck \
  && cp ./target/"${RUST_TARGET}"/prod/vym-fyi-server-crud vym-fyi-crud \
  && cp ./target/"${RUST_TARGET}"/prod/vym-fyi-server-redirect vym-fyi-redirect \
  && cp ./target/"${RUST_TARGET}"/prod/vym-fyi-healthcheck healthcheck

FROM gcr.io/distroless/static-debian12:nonroot as baseprod

LABEL maintainer="Stephane Segning <selastlambou@gmail.com>"
LABEL org.opencontainers.image.description="vymalo"

FROM baseprod as crud

ENV RUST_LOG=warn
ENV PORT=8000

WORKDIR /app

COPY --from=builder /app/vym-fyi-crud /app/vym-fyi-crud
COPY --from=builder /app/healthcheck /app/healthcheck
COPY static /app/static

USER nonroot:nonroot

EXPOSE $PORT

HEALTHCHECK --interval=10s --timeout=3s --start-period=2s --retries=5 CMD ["/app/healthcheck", "--port", "8000", "--path", "/health"]

ENTRYPOINT ["/app/vym-fyi-crud"]

FROM baseprod as redirect

ENV RUST_LOG=warn
ENV PORT=8000

WORKDIR /app

COPY --from=builder /app/vym-fyi-redirect /app/vym-fyi-redirect
COPY --from=builder /app/healthcheck /app/healthcheck
COPY static /app/static

USER nonroot:nonroot

EXPOSE $PORT

HEALTHCHECK --interval=10s --timeout=3s --start-period=2s --retries=5 CMD ["/app/healthcheck", "--port", "8000", "--path", "/health"]

ENTRYPOINT ["/app/vym-fyi-redirect"]


FROM baseprod as crud_k8s

ENV RUST_LOG=warn
ENV PORT=8000

WORKDIR /app

COPY --from=builder /app/vym-fyi-crud /app/vym-fyi-crud
COPY static /app/static

USER nonroot:nonroot

EXPOSE $PORT

ENTRYPOINT ["/app/vym-fyi-crud"]

FROM baseprod as redirect_k8s

ENV RUST_LOG=warn
ENV PORT=8000

WORKDIR /app

COPY --from=builder /app/vym-fyi-redirect /app/vym-fyi-redirect
COPY static /app/static

USER nonroot:nonroot

EXPOSE $PORT

ENTRYPOINT ["/app/vym-fyi-redirect"]
