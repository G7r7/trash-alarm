FROM rust:1.63.0-slim-bullseye

# Dev tools
RUN apt update \
    && apt-get -y install git \
    && apt-get -y install wget \
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

# SSH server for remote coding with IntelliJ
EXPOSE 22
RUN mkdir /var/run/sshd
RUN echo 'PermitRootLogin yes' >> /etc/ssh/sshd_config
RUN echo 'PasswordAuthentication yes' >> /etc/ssh/sshd_config
RUN echo 'PermitEmptyPasswords yes' >> /etc/ssh/sshd_config

# Install sudo
RUN apt-get install sudo    

# Service mangement with supervisor
RUN apt-get install -y supervisor
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Change root password.
RUN echo 'root:root' | chpasswd

# Dev env user
# RUN addgroup --gid 1000 devuser
# RUN adduser --disabled-password --gecos "" --uid 1000 --gid 1000 devuser
# RUN usermod -aG sudo devuser
# RUN echo 'devuser ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers
# RUN passwd -d devuser
# ENV HOME /home/devuser
# USER devuser

# Export rust environment variables for Rust IntelliJ over SSH.
RUN echo "export RUST_VERSION=${RUST_VERSION}" >> ~/.bashrc
RUN echo "export RUSTUP_HOME=${RUSTUP_HOME}" >> ~/.bashrc
RUN echo "export CARGO_HOME=${CARGO_HOME}" >> ~/.bashrc
RUN echo "export PATH=${PATH}" >> ~/.bashrc

# Creating mount directory.
RUN mkdir /mnt/pico
