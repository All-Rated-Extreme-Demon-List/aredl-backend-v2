FROM rust:1.85 AS build

WORKDIR /usr/src/seed_aredl
RUN apt-get update && apt-get install -y libpq-dev

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

COPY --from=build /usr/lib/x86_64-linux-gnu/libpq.so* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libgssapi_krb5* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libldap* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libkrb5* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libk5crypto* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libkrb5support* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/liblber* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libsasl* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libgnutls* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libp11* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libidn* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libunistring* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libtasn1* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libnettle* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libhogweed* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libgmp* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libffi* /usr/lib/
COPY --from=build /lib/x86_64-linux-gnu/libcom_err* /lib/
COPY --from=build /lib/x86_64-linux-gnu/libkeyutils* /lib/

COPY tools/seed_aredl/migrate.sh /usr/local/bin/migrate.sh
RUN chmod +x /usr/local/bin/migrate.sh


CMD ["migrate.sh"]