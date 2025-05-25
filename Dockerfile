# multi-stage build

FROM ghcr.io/rust-cross/rust-musl-cross:aarch64-musl as builder
ENV SQLX_OFFLINE=true

WORKDIR /app


# 安装必要的依赖
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    pkg-config \
    musl-tools \
    build-essential \
    wget \
    linux-libc-dev

RUN rustup -V

#copy the code
COPY ./chat ./chat
COPY ./protos ./protos

# 编译并安装 musl 版本的 OpenSSL
RUN wget https://www.openssl.org/source/openssl-3.0.8.tar.gz \
    && tar xvf openssl-3.0.8.tar.gz \
    && cd openssl-3.0.8 \
    && CC=aarch64-linux-musl-gcc ./Configure no-shared no-zlib no-secure-memory no-afalgeng no-legacy -fPIC --prefix=/usr/local/musl/aarch64-unknown-linux-musl linux-aarch64 \
    && make build_libs \
    && cp -a include/openssl /usr/local/musl/aarch64-unknown-linux-musl/include/ \
    && cp -a libcrypto.a libssl.a /usr/local/musl/aarch64-unknown-linux-musl/lib/ \
    && cd .. \
    && rm -rf openssl-3.0.8.tar.gz openssl-3.0.8
#build the code
ENV CARGO_BUILD_JOBS=1
ENV PKG_CONFIG_ALLOW_CROSS=1 \
    OPENSSL_DIR=/usr/local/musl/aarch64-unknown-linux-musl \
    OPENSSL_LIB_DIR=/usr/local/musl/aarch64-unknown-linux-musl/lib \
    OPENSSL_INCLUDE_DIR=/usr/local/musl/aarch64-unknown-linux-musl/include \
    PKG_CONFIG_PATH=/usr/local/musl/aarch64-unknown-linux-musl/lib/pkgconfig
ENV ORT_LIB_LOCATION=/app/onnxruntime-aarch64-musl
RUN cd chat && cargo build --release --target aarch64-unknown-linux-musl
RUN ls  /app/chat/target/aarch64-unknown-linux-musl/release


#build the image
FROM alpine:3.20
WORKDIR /app
ARG APP_NAME
ARG APP_PORT
# Create a non-root user and group
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

# Copy the binary and set permissions
COPY --from=builder /app/chat/target/aarch64-unknown-linux-musl/release/${APP_NAME} /app/${APP_NAME}
RUN chmod +x /app/${APP_NAME} && \
    chown -R appuser:appgroup /app

# Switch to non-root user
USER appuser



ENTRYPOINT /app/${APP_NAME}
EXPOSE ${APP_PORT}
