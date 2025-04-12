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

FROM gcr.io/distroless/cc-debian12

ARG BUILD_PROFILE=debug
COPY --from=build /usr/src/aredl-backend/target/${BUILD_PROFILE}/aredl-backend /usr/local/bin/aredl-backend

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

CMD ["aredl-backend"]
