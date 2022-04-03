FROM rust:latest
WORKDIR /app
COPY . .

RUN apt-get update && \
    apt-get install -y vim

