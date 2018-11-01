# This file is part of yamba which is released under <GPL3>. See file LICENSE or go to https://www.gnu.org/licenses/gpl.html for full license details.

FROM debian:stable-slim

LABEL maintainers="Aron Heinecke <Ox0p54r36@t-online.de>, \
                   Oliver Habersetzer <oliver.habersetzer@gmail.com>, \
                   Tristan Schönhals <tristan.schoenhals@gmail.com>"

EXPOSE 80

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update -q && apt-get install -qqy \
    # Install tools
    vim \
    wget \
    net-tools \
    apt-transport-https \
    gnupg2 \
    # Install nginx
    nginx \
    && wget -q https://packages.sury.org/php/apt.gpg -O- | APT_KEY_DONT_WARN_ON_DANGEROUS_USAGE=DontWarn apt-key add - \
    && echo "deb https://packages.sury.org/php/ stretch main" | tee /etc/apt/sources.list.d/php.list \
    && apt-get update -q \
    && apt-get install -qqy \
    # Install php
    php7.2-fpm \
    php7.2-mbstring \
    php7.2-intl \
    php7.2-xml \
    php7.2-mysql \
    php7.2-zip \
    composer \
    # Delete apt cache
    && apt-get autoremove \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /var/www/html

# Add config files
COPY public_html/index.php ./
COPY public_html/bin ./bin
COPY nginx.conf /etc/nginx/sites-available/default
COPY public_html/composer.json ./
COPY public_html/composer.lock ./
COPY public_html/config ./config

RUN COMPOSER_ALLOW_SUPERUSER=1 composer install -n \
    # Define logsDir
    && cakeLogsDir=/var/log/cake/logs \
    # Override app config with default
    cp ./config/app.default.php ./config/app.php \
    # Create and configure logs and tmp directory
    && mkdir -p cakeLogsDir tmp \
    && chgrp -R www-data cakeLogsDir tmp \
    && chmod -R g+rw cakeLogsDir tmp \
    && ln -s cakeLogsDir logs

# Add source files
COPY public_html/src ./src
COPY public_html/webroot ./webroot
COPY public_html/copyEnvVars.sh .

CMD /bin/bash -c ./copyEnvVars.sh && service php7.2-fpm start && nginx -g "daemon off;"