# Builder stage
FROM rockylinux:9 AS builder

# Install build dependencies and Russian locale support
RUN dnf install -y \
    gcc \
    gcc-c++ \
    openssl-devel \
    pkgconf \
    glibc-langpack-ru \
    && dnf clean all

# Install latest Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.93.0

# Add cargo/rust to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Set Russian locale (note: RHEL uses lowercase 'utf8' in locale names)
ENV LANG=ru_RU.utf8 \
    LANGUAGE=ru_RU:ru \
    LC_ALL=ru_RU.utf8

# Copy source code
WORKDIR /app
COPY . .

ARG BOOSTY=false

# Build the binary
RUN if [ "$BOOSTY" = "true" ]; then \
        cargo build --release --features boosty; \
    else \
        cargo build --release; \
    fi

# Runtime stage
FROM rockylinux:9-minimal

# Install minimal runtime dependencies:
# - openssl-libs: for native-tls (used by teloxide/reqwest in default config)
RUN microdnf install -y \
    openssl-libs \
    ca-certificates \
    && microdnf clean all

# Set default locale
ENV LANG=ru_RU.utf8 \
    LANGUAGE=ru_RU:ru \
    LC_ALL=ru_RU.utf8

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/emtt /usr/local/bin/emtt

# Expose UDP port 50514 (for syslog)
EXPOSE 50514/udp

# Run the app (defaults to syslog subcommand; args via env vars)
ENTRYPOINT ["emtt"]
CMD ["syslog"]
