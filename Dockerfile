FROM rust:latest
WORKDIR /app

ARG TOOLCHAIN_URL=https://developer.arm.com/-/media/Files/downloads/gnu/12.2.rel1/binrel/arm-gnu-toolchain-12.2.rel1-x86_64-arm-none-linux-gnueabihf.tar.xz

RUN apt-get update && \
    apt-get install -y vim && \
    wget ${TOOLCHAIN_URL}

RUN tar -xf arm-gnu-toolchain-12.2.rel1-x86_64-arm-none-linux-gnueabihf.tar.xz

ENV PATH "$PATH:/app/arm-gnu-toolchain-12.2.rel1-x86_64-arm-none-linux-gnueabihf/bin"

COPY tests/asm/and.asm .

RUN mkdir /app/out

# CMD exec arm-none-linux-gnueabihf-as -o /dev/nul -al --no-warn > /app/out/out.txt and.asm

CMD ["arm-none-linux-gnueabihf-as", "-o", "/dev/nul", "-al", "--no-warn", "-mbig-endian"]

