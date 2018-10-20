#!/bin/bash
git pull && version=$(git rev-parse HEAD) || version=testing;
echo "Version=${version}";
sudo docker rm -f yamba 2>/dev/null;
cd public_html;
composer install -n;
cd ..;
sudo docker build -t yamba:${version} .;
sudo docker run -d --net="host" --name yamba -p 80:80 yamba:${version};