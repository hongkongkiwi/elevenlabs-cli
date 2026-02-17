FROM alpine:3.19

RUN apk add --no-cache alsa-lib openssl ca-certificates

COPY docker-bin/elevenlabs-cli-amd64 /usr/bin/elevenlabs-cli-x86_64
COPY docker-bin/elevenlabs-cli-arm64 /usr/bin/elevenlabs-cli-aarch64

RUN arch=$(uname -m) && \
    case "$arch" in \
        x86_64) cp /usr/bin/elevenlabs-cli-x86_64 /usr/bin/elevenlabs-cli ;; \
        aarch64) cp /usr/bin/elevenlabs-cli-aarch64 /usr/bin/elevenlabs-cli ;; \
        *) echo "Unsupported architecture: $arch" && exit 1 ;; \
    esac && \
    chmod +x /usr/bin/elevenlabs-cli && \
    rm -f /usr/bin/elevenlabs-cli-*

ENTRYPOINT ["/usr/bin/elevenlabs-cli"]
