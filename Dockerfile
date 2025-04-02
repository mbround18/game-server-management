# Stage 1: Base Image
FROM rust:1.85 AS base

# Install necessary build tools
RUN apt-get update && apt-get install -y cmake

# Create a non-root user for build steps
RUN useradd -m rustuser
USER rustuser
WORKDIR /home/rustuser/app

# Stage 2: Planner
FROM base AS planner
# Copy the entire repository so that all workspace members (e.g., libs/*) are present.
COPY --chown=rustuser:rustuser . .
# Install cargo-chef and prepare the recipe.
RUN cargo install cargo-chef && \
    cargo chef prepare --recipe-path recipe.json

# Stage 3: Cacher
FROM base AS cacher
WORKDIR /home/rustuser/app
# Copy the recipe from the planner stage.
COPY --chown=rustuser:rustuser --from=planner /home/rustuser/app/recipe.json .
# Install cargo-chef and cache dependencies.
RUN cargo install cargo-chef && \
    cargo chef cook --release --recipe-path recipe.json

# Stage 4: Builder
FROM base AS builder
WORKDIR /home/rustuser/app
# Copy the full repository.
COPY --chown=rustuser:rustuser . .
# Copy cached dependencies from the cacher stage.
COPY --from=cacher /home/rustuser/app/target target
COPY --from=cacher /usr/local/cargo/registry /usr/local/cargo/registry
# Build all binaries in release mode.
RUN cargo build --release

# Stage 5: Runtime
FROM debian:12-slim AS gh-runtime
# Install any runtime dependencies (adjust as needed)
RUN apt-get update && apt-get install -y libssl-dev curl jq unzip && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --chmod=0755 ./dist .

CMD ["bash"]


FROM debian:12-slim AS runtime
# Install any runtime dependencies (adjust as needed)
RUN apt-get update && apt-get install -y libssl-dev && \
    apt-get clean && rm -rf /var/lib/apt/lists/*
WORKDIR /app
# Copy all compiled binaries (and any other artifacts) from the builder stage.
COPY --from=builder /home/rustuser/app/target/release/ .
# By default, we start a shell so you can choose which binary to run.
CMD ["bash"]
