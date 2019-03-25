FROM ubuntu:18.10 as toolchain

LABEL maintainers="Aron Heinecke <Ox0p54r36@t-online.de>, \
                   Tristan Schönhals <tristan.schoenhals@gmail.com>"

EXPOSE 1338

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update -q && apt-get install -qqy \
    # Install tools
    nano \
    net-tools \
    libasound2-dev \
    xvfb \
    python3 \
    libnss3 \
    curl \
    sudo \
    gcc \
    pkg-config \
    # ssl dev
    libssl-dev \
    # pulse dev
    libpulse-dev \
    pulseaudio \
    # Install gstreamer requirements
    libgstreamer1.0-0 gstreamer1.0-pulseaudio \
    libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav libgstrtspserver-1.0-dev \
    libgstreamer-plugins-bad1.0-dev \
    libglib2.0-dev \
    # ?
    sqlite3 \
    libsqlite3-dev \
    # cleanup
    && apt-get autoremove \
    && rm -rf /var/lib/apt/lists/* \
    # fixup: python3 -> python softlink
    && ln -s $(which python3) /usr/local/bin/python \
    # install rust
    && curl -sf -L https://static.rust-lang.org/rustup.sh | sh -s -- -y

FROM toolchain as dependencies

# create dummy projects
WORKDIR /opt/yamba
RUN USER=root ~/.cargo/bin/cargo new --lib --vcs none ts3plugin
RUN USER=root ~/.cargo/bin/cargo new --bin --vcs none daemon

# types
WORKDIR /opt/types
COPY ./types ./
RUN ~/.cargo/bin/cargo build

# build daemon dependencies
WORKDIR /opt/yamba/daemon
COPY ./daemon/Cargo.* ./
RUN ~/.cargo/bin/cargo build && rm ./src/*.rs ./target/debug/deps/yamba_daemon*

# build ts3plugin dependencies
WORKDIR /opt/yamba/ts3plugin
COPY ./ts3plugin/Cargo.* ./
RUN ~/.cargo/bin/cargo build && rm ./src/*.rs ./target/debug/deps/libts3plugin*

FROM dependencies as application

# build ts3plugin
WORKDIR /opt/yamba/ts3plugin
COPY ./ts3plugin ./
RUN ~/.cargo/bin/cargo build

# build daemon
WORKDIR /opt/yamba/daemon
COPY ./daemon ./
RUN ~/.cargo/bin/cargo build

WORKDIR /opt/yamba

COPY ./daemon/templates/* /root/templates/

# copy entrypoint
COPY ./entrypoint.sh ./entrypoint.sh
RUN chmod 755 ./entrypoint.sh

ENTRYPOINT ["./entrypoint.sh"]
CMD ["/opt/yamba/daemon/target/debug/yamba-daemon"]
