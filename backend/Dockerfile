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
    # install rust
    && curl -sf -L https://static.rust-lang.org/rustup.sh | sh -s -- -y

# copy entrypoint
COPY ./entrypoint.sh /entrypoint.sh
RUN chmod 755 /entrypoint.sh
    
# copy daemon
COPY ./daemon/resources /opt/yamba/daemon/resources
COPY ./daemon/src /opt/yamba/daemon/src
COPY ./daemon/conf /opt/yamba/daemon/conf
COPY ./daemon/default_log.yml /opt/yamba/daemon/default_log.yml
COPY ./daemon/Cargo.toml /opt/yamba/daemon/Cargo.toml

# copy ts3plugin
COPY ./ts3plugin/src /opt/yamba/ts3plugin/src
COPY ./ts3plugin/Cargo.toml /opt/yamba/ts3plugin/Cargo.toml

# build dependencies
RUN ~/.cargo/bin/cargo install cargo-build-deps
WORKDIR /opt/yamba/daemon
RUN ~/.cargo/bin/cargo build-deps --release
WORKDIR /opt/yamba/ts3plugin
RUN ~/.cargo/bin/cargo build-deps --release

# build projects
WORKDIR /opt/yamba/daemon
RUN ~/.cargo/bin/cargo build
WORKDIR /opt/yamba/ts3plugin
RUN ~/.cargo/bin/cargo build

WORKDIR /opt/yamba/daemon/target/debug
# fixup: python3 -> python softlink
RUN ln -s $(which python3) /usr/local/bin/python

ENTRYPOINT ["/entrypoint.sh"]
#CMD ["/opt/yamba/daemon/target/debug/yamba-backend"]
CMD ["/opt/yamba/daemon/target/debug/yamba-backend", "test-ts","-h","ek.proctet.net","--cid","5368","--clear-instances"]
