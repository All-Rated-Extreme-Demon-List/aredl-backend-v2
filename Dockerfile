FROM rust:1.93 AS build

ARG BUILD_PROFILE=debug
ARG TARGETARCH
WORKDIR /usr/src/aredl-backend
RUN apt-get update && apt-get install -y libpq-dev

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

FROM debian:13-slim

ARG BUILD_PROFILE=debug

RUN apt-get update && apt-get install -y --no-install-recommends libpq-dev exiftool ca-certificates

COPY --from=build /usr/src/aredl-backend/target/${BUILD_PROFILE}/aredl-backend /usr/local/bin/aredl-backend
COPY --from=ffbin /ffprobe /usr/local/bin/ffprobe

CMD ["aredl-backend"]
