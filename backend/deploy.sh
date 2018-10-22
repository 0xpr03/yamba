#!/bin/bash
git pull && version=$(git rev-parse HEAD) || version=testing;
echo "Version=${version}";
echo "Building executable... ";
cd ./deamon/
RUST_BACKTRACE=1 cargo build --release
cd ..
cd ./ts3plugin//
RUST_BACKTRACE=1 cargo build --release
cd ..
if [ ! -d "ts3client" ]; then
    chmod +x ./ts3-dl.sh
    ./ts3-dl.sh
fi
sudo docker rm -f yamba 2>/dev/null;
sudo docker build -t yamba:${version} .;
sudo docker run -v $(realpath ./ts3client):/opt/ts3 --net="host" --name yamba -p 80:80 yamba:${version};