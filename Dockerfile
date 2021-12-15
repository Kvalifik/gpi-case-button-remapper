FROM rust:latest

WORKDIR /usr/src/app

RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt-get upgrade -qq && \
    apt-get install -qq \
    gcc-arm-linux-gnueabihf \
    libxdo-dev:armhf \
    xdotool:armhf

RUN rustup target add armv7-unknown-linux-gnueabihf

CMD ["sleep", "infinity"]

