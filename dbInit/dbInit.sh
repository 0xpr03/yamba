#!/bin/bash
# This file is part of yamba which is released under <GPL3>. See file LICENSE or go to https://www.gnu.org/licenses/gpl.html for full license details.

for file in /docker-entrypoint-initdb.d/schema/*.sql; do
    echo $file;
    mysql --user=root --password=$MYSQL_ROOT_PASSWORD < $file $MYSQL_DATABASE;
done

if [[ $YAMBA_DEBUG == true ]]; then
    echo Creating debug_kit;
    mysql --user=root --password=$MYSQL_ROOT_PASSWORD -e "CREATE DATABASE debug_kit" $MYSQL_DATABASE;
fi

echo Setting up backend user permissions;
mysql --user=root --password=$MYSQL_ROOT_PASSWORD -e "
REVOKE ALL PRIVILEGES ON ${MYSQL_DATABASE}.* FROM $MYSQL_USER;
GRANT ALL PRIVILEGES ON ${MYSQL_DATABASE}.streams TO $MYSQL_USER;
GRANT ALL PRIVILEGES ON ${MYSQL_DATABASE}.songs TO $MYSQL_USER;
GRANT ALL PRIVILEGES ON ${MYSQL_DATABASE}.playlists TO $MYSQL_USER;
GRANT ALL PRIVILEGES ON ${MYSQL_DATABASE}.songs_to_playlists TO $MYSQL_USER;
GRANT SELECT ON ${MYSQL_DATABASE}.permissions TO $MYSQL_USER;
GRANT SELECT ON ${MYSQL_DATABASE}.permission_groups TO $MYSQL_USER;
GRANT SELECT ON ${MYSQL_DATABASE}.permissions_to_permission_groups TO $MYSQL_USER;
GRANT SELECT ON ${MYSQL_DATABASE}.users_to_permission_groups TO $MYSQL_USER;
GRANT SELECT ON ${MYSQL_DATABASE}.ts3_groups TO $MYSQL_USER;
GRANT SELECT ON ${MYSQL_DATABASE}.ts3_groups_to_permission_groups TO $MYSQL_USER;
FLUSH PRIVILEGES;
" $MYSQL_DATABASE;