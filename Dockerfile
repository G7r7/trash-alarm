FROM rust:1.63.0-slim-bullseye

# Dev tools
RUN apt update \
    && apt-get -y install git \
    && apt-get -y install openssh-client \
    && apt-get -y install openssh-server \
    && git config --global core.autocrlf true \
    && git config --global --add safe.directory /app

# Mandatory tools
RUN apt install -y pkg-config libudev-dev

# Rust packages
RUN rustup self update
RUN rustup update stable
RUN rustup target add thumbv6m-none-eabi
RUN rustup component add rustfmt
RUN cargo install flip-link
# Useful to creating UF2 images for the RP2040 USB Bootloader
RUN cargo install elf2uf2-rs --locked
# Useful for flashing over the SWD pins using a supported JTAG probe
RUN cargo install probe-run

# SSH server
EXPOSE 22
RUN mkdir /var/run/sshd
RUN echo 'root:root' | chpasswd
RUN sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config

# Dev env user
RUN addgroup --gid 1000 devuser
RUN adduser --gecos "" --uid 1000 --gid 1000 devuser
RUN echo 'devuser:devuser' | chpasswd
ENV HOME /home/devuser
USER devuser