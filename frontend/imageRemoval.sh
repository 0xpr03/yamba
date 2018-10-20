#!/bin/bash
sudo docker rmi $(sudo docker images | grep "^yamba\|^<none>" | awk '{print $3}');