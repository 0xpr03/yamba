#!/bin/bash
for file in /docker-entrypoint-initdb.d/schema/*.sql; do
    echo $file;
    mysql --user=root --password=$MYSQL_ROOT_PASSWORD < $file $MYSQL_DATABASE;
done
[[ $YAMBA_DEBUG == true ]] && mysql --user=root --password=$MYSQL_ROOT_PASSWORD -e "CREATE DATABASE debug_kit" $MYSQL_DATABASE;
mysql --user=root --password=$MYSQL_ROOT_PASSWORD -e "
REVOKE ALL PRIVILEGES ON ${MYSQL_DATABASE}.* FROM $MYSQL_USER;
GRANT ALL PRIVILEGES ON ${MYSQL_DATABASE}.streams TO $MYSQL_USER;
GRANT ALL PRIVILEGES ON ${MYSQL_DATABASE}.songs TO $MYSQL_USER;
GRANT ALL PRIVILEGES ON ${MYSQL_DATABASE}.playlists TO $MYSQL_USER;
GRANT ALL PRIVILEGES ON ${MYSQL_DATABASE}.songs2playlists TO $MYSQL_USER;
FLUSH PRIVILEGES;
" $MYSQL_DATABASE;