# Prompter Docker 构建文件
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .

# 安装依赖并编译
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release

# 运行镜像
FROM ubuntu:22.04

WORKDIR /app

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/prompter /usr/local/bin/
COPY --from=builder /app/docker-entrypoint.sh /usr/local/bin/

# 创建数据目录
RUN mkdir -p /data
VOLUME ["/data"]

ENTRYPOINT ["docker-entrypoint.sh"]
CMD ["--simple"]