FROM ubuntu:18.10

LABEL maintainers="Aron Heinecke <Ox0p54r36@t-online.de>, \
                   Oliver Habersetzer <oliver.habersetzer@gmail.com>, \
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
    # install rust
    && curl -sf -L https://static.rust-lang.org/rustup.sh | sh

# copy entrypoint
COPY ./entrypoint.sh /entrypoint.sh
RUN chmod 755 /entrypoint.sh
    
# copy daemon
COPY ./daemon/src /opt/yamba/daemon/src
COPY ./daemon/conf /opt/yamba/daemon/conf
COPY ./daemon/default_log.yml /opt/yamba/daemon/default_log.yml
COPY ./daemon/Cargo.toml /opt/yamba/daemon/Cargo.toml
WORKDIR /opt/yamba/daemon
RUN cargo build

# copy ts3plugin
COPY ./ts3plugin/src /opt/yamba/ts3plugin/src
COPY ./ts3plugin/Cargo.toml /opt/yamba/ts3plugin/Cargo.toml
WORKDIR /opt/yamba/ts3plugin
RUN cargo build

WORKDIR /opt/yamba/daemon/target/debug
RUN ln -s $(which python3) /usr/local/bin/python

ENTRYPOINT ["/entrypoint.sh"]
CMD "/opt/yamba/daemon/target/debug/yamba-backend"
