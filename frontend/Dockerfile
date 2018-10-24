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
    php7.0-fpm \
    php7.0-mbstring \
    php7.0-intl \
    php7.0-xml \
    php7.0-mysql \
    php7.0-zip \
    php7.0-sqlite3 \
    # Install nginx
    nginx \
    # Delete apt cache
    && apt-get autoremove \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /var/www/html

COPY public_html/index.php ./
COPY public_html/bin ./bin
COPY public_html/vendor ./vendor
COPY public_html/config ./config
COPY nginx.conf /etc/nginx/sites-available/default

RUN cakeLogsDir=/var/log/cake/logs \
    # Override app config with default
    cp ./config/app.default.php ./config/app.php \
    # Configure random salt (hashing value for cake)
    && sed -i -e "s/__SALT__/$(date +%s | sha256sum | base64 | head -c 64 ; echo)/" config/app.php \
    # Create and configure logs and tmp directory
    && mkdir -p cakeLogsDir tmp \
    && chgrp -R www-data cakeLogsDir tmp \
    && chmod -R g+rw cakeLogsDir tmp \
    && ln -s cakeLogsDir logs

COPY public_html/src ./src
COPY public_html/webroot ./webroot

CMD service php7.0-fpm start && nginx -g "daemon off;"