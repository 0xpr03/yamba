#!/bin/bash
git pull && version=$(git rev-parse HEAD) || version=testing;
echo "Version=${version}";
sudo docker rm -f yamba 2>/dev/null;
sudo docker build -t yamba:${version} .;
sudo docker run -d --name yamba -p 80:80 yamba:${version};