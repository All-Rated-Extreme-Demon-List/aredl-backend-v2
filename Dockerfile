FROM rust:1.86 AS build

ARG BUILD_PROFILE=debug
ARG TARGETARCH
WORKDIR /usr/src/aredl-backend
RUN apt-get update && apt-get install -y libpq-dev

RUN if [ "$TARGETARCH" = "arm64" ]; then \
      ln -s /usr/lib/aarch64-linux-gnu /usr/lib/libdir && \
      ln -s /lib/aarch64-linux-gnu /lib/libdir; \
    else \
      ln -s /usr/lib/x86_64-linux-gnu /usr/lib/libdir && \
      ln -s /lib/x86_64-linux-gnu /lib/libdir; \
    fi

COPY Cargo.toml Cargo.lock diesel.toml ./
RUN mkdir src && echo 'fn main() {println!("This is a dummy file.")}' > src/main.rs
RUN if [ "$BUILD_PROFILE" = "release" ]; then \
      cargo build --release; \
    else \
      cargo build; \
    fi

COPY src/ ./src/
COPY migrations/ ./migrations/
RUN touch ./src/main.rs

RUN if [ "$BUILD_PROFILE" = "release" ]; then \
      cargo build --release; \
    else \
      cargo build; \
    fi

FROM mwader/static-ffmpeg:8.0 AS ffbin

FROM debian:12-slim

ARG BUILD_PROFILE=debug

RUN apt-get update && apt-get install -y --no-install-recommends libpq-dev exiftool ca-certificates

COPY --from=build /usr/src/aredl-backend/target/${BUILD_PROFILE}/aredl-backend /usr/local/bin/aredl-backend
COPY --from=ffbin /ffprobe /usr/local/bin/ffprobe

CMD ["aredl-backend"]
