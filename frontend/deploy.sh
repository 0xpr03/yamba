#!/bin/bash
git pull && version=$(git rev-parse HEAD) || version=testing;
echo "Version=${version}";
sudo docker rm -f yamba_frontend 2>/dev/null;
sudo docker build -t yamba_frontend:${version} .;
sudo docker run -d --link="yamba_mariadb:database" --name yamba_frontend -p 80:80 yamba_frontend:${version};