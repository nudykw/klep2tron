# Базовый Dockerfile для игрового сервера
FROM rust:1.80-slim as builder

WORKDIR /usr/src/kleptotron
COPY . .

# Установка зависимостей (если потребуется pkg-config, udev и тд для headless bevy)
RUN apt-get update && apt-get install -y pkg-config libudev-dev

# Собираем только сервер в релизе
RUN cargo build --release -p server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/kleptotron/target/release/server /usr/local/bin/server

EXPOSE 5000/udp
CMD ["server"]
