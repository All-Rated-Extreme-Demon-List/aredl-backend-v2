FROM rust:1.85 AS build

ARG TARGETARCH

WORKDIR /usr/src/seed_aredl
RUN apt-get update && apt-get install -y libpq-dev

RUN if [ "$TARGETARCH" = "arm64" ]; then \
      ln -s /usr/lib/aarch64-linux-gnu /usr/lib/libdir && \
      ln -s /lib/aarch64-linux-gnu /lib/libdir; \
    else \
      ln -s /usr/lib/x86_64-linux-gnu /usr/lib/libdir && \
      ln -s /lib/x86_64-linux-gnu /lib/libdir; \
    fi
	
COPY tools/seed_aredl/Cargo.toml tools/seed_aredl/Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY migrations ./migrations
COPY tools/seed_aredl/src ./src
RUN touch src/main.rs
RUN cargo build --release

FROM debian:12

RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/src/seed_aredl/target/release/seed_aredl /usr/local/bin/seed_aredl

COPY --from=build /usr/lib/libdir/libpq.so* /usr/lib/
COPY --from=build /usr/lib/libdir/libgssapi_krb5* /usr/lib/
COPY --from=build /usr/lib/libdir/libldap* /usr/lib/
COPY --from=build /usr/lib/libdir/libkrb5* /usr/lib/
COPY --from=build /usr/lib/libdir/libk5crypto* /usr/lib/
COPY --from=build /usr/lib/libdir/libkrb5support* /usr/lib/
COPY --from=build /usr/lib/libdir/liblber* /usr/lib/
COPY --from=build /usr/lib/libdir/libsasl* /usr/lib/
COPY --from=build /usr/lib/libdir/libgnutls* /usr/lib/
COPY --from=build /usr/lib/libdir/libp11* /usr/lib/
COPY --from=build /usr/lib/libdir/libidn* /usr/lib/
COPY --from=build /usr/lib/libdir/libunistring* /usr/lib/
COPY --from=build /usr/lib/libdir/libtasn1* /usr/lib/
COPY --from=build /usr/lib/libdir/libnettle* /usr/lib/
COPY --from=build /usr/lib/libdir/libhogweed* /usr/lib/
COPY --from=build /usr/lib/libdir/libgmp* /usr/lib/
COPY --from=build /usr/lib/libdir/libffi* /usr/lib/
COPY --from=build /lib/libdir/libcom_err* /lib/
COPY --from=build /lib/libdir/libkeyutils* /lib/

COPY tools/seed_aredl/migrate.sh /usr/local/bin/migrate.sh

RUN sed -i 's/\r$//' /usr/local/bin/migrate.sh
RUN chmod +x /usr/local/bin/migrate.sh

CMD ["migrate.sh"]