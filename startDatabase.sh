#!/bin/bash
sudo docker rm -f yamba_mariadb 2>/dev/null;
sudo docker run --name yamba_mariadb -e MYSQL_ROOT_PASSWORD=$MYSQL_ROOT_PASSWORD -d mariadb:10.3.10-bionic;