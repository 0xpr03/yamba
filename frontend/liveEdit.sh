#!/bin/bash
sudo docker rm -f yamba 2>/dev/null;
sudo docker run -v $(realpath ./public_html):/var/www/html -d --net="host" --name yamba -p 80:80 yamba:$(sudo docker images | grep "^yamba" | awk ' {print $2}' | head -n 1);
