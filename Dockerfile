FROM node:24-bookworm-slim AS frontend
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY index.html vite.config.js ./
COPY resources ./resources
COPY public ./public
RUN npm run build

FROM rust:slim-bookworm AS backend
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs main.rs ./
COPY app ./app
COPY bootstrap ./bootstrap
COPY config ./config
COPY database ./database
COPY routes ./routes
COPY --from=frontend /app/dist ./dist
COPY desktop ./desktop
RUN cargo build --locked --release --bin forge --features embedded-assets

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl libssl3 && rm -rf /var/lib/apt/lists/* && useradd --create-home app
WORKDIR /app
COPY --from=backend /app/target/release/forge /usr/local/bin/forge
RUN mkdir storage && chown app:app storage
USER app
ENV APP_HOST=0.0.0.0 APP_PORT=3000
EXPOSE 3000
CMD ["forge", "serve"]
