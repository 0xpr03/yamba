# Frontend for website access

**Do NOT run composer commands, dockerfile takes care of that**

Otherwise you would screw over the composer.lock

## Required for development:
docker and docker-compose

## When using liveedit:
You need to exec into your container and execute `composer install` the first time
This should not be necessary on redeploys.