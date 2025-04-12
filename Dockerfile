FROM rust:1.86 AS build

ARG BUILD_PROFILE=debug
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

FROM gcr.io/distroless/cc-debian12

ARG BUILD_PROFILE=debug
COPY --from=build /usr/src/aredl-backend/target/${BUILD_PROFILE}/aredl-backend /usr/local/bin/aredl-backend

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

CMD ["aredl-backend"]
