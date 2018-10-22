FROM debian:stable-slim

LABEL maintainers="Aron Heinecke <Ox0p54r36@t-online.de>, \
                   Oliver Habersetzer <oliver.habersetzer@gmail.com>, \
                   Tristan Schönhals <tristan.schoenhals@gmail.com>"

EXPOSE 80

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update -q && apt-get install -qqy \
    # Install tools
    vim \
    # Install php
    libasound2-dev \
    xvfb \
    vlc \
    python3 \
    && apt-get autoremove \
    && rm -rf /var/lib/apt/lists/*
    
WORKDIR /var/yamba

COPY ./deamon/target/release/yamba-backend ./
COPY ./ts3plugin/target/release/libyamba_plugin.so ~/.ts3client/plugins/
COPY ./ts3-start.sh ./

CMD /var/yamba/yamba-backend