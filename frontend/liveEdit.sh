#!/bin/bash
sudo docker rm -f yamba_frontend 2>/dev/null;
sudo docker run -v $(realpath ./public_html):/var/www/html -d --net="host" --name yamba_frontend -p 80:80 yamba_frontend:$(sudo docker images | grep "^yamba_frontend" | awk ' {print $2}' | head -n 1);
