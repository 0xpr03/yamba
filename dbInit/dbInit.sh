#!/bin/bash
# This file is part of yamba which is released under <GPL3>. See file LICENSE or go to https://www.gnu.org/licenses/gpl.html for full license details.

# Create queuesadilla database
mysql --user=root --password=$YAMBA_DATABASE_ROOT_PASSWORD -e "CREATE DATABASE queuesadilla";

for file in /docker-entrypoint-initdb.d/schema/*.sql; do
    echo $file;
    mysql --user=root --password=$YAMBA_DATABASE_ROOT_PASSWORD < $file $([[ $(echo $file | grep -v "_vendor.sql$") ]] && echo $YAMBA_DATABASE_DATABASE || echo queuesadilla);
done

if [[ $YAMBA_DEBUG == true ]]; then
    echo Creating debug_kit;
    mysql --user=root --password=$YAMBA_DATABASE_ROOT_PASSWORD -e "CREATE DATABASE debug_kit" $YAMBA_DATABASE_DATABASE;
fi

echo Setting up backend user permissions;
mysql --user=root --password=$YAMBA_DATABASE_ROOT_PASSWORD -e "
REVOKE ALL PRIVILEGES ON ${YAMBA_DATABASE_DATABASE}.* FROM $YAMBA_DATABASE_USER;
GRANT ALL PRIVILEGES ON ${YAMBA_DATABASE_DATABASE}.titles TO $YAMBA_DATABASE_USER;
GRANT ALL PRIVILEGES ON ${YAMBA_DATABASE_DATABASE}.instances TO $YAMBA_DATABASE_USER;
GRANT SELECT ON ${YAMBA_DATABASE_DATABASE}.playlists TO $YAMBA_DATABASE_USER;
GRANT SELECT ON ${YAMBA_DATABASE_DATABASE}.titles_to_playlists TO $YAMBA_DATABASE_USER;
GRANT SELECT ON ${YAMBA_DATABASE_DATABASE}.permissions TO $YAMBA_DATABASE_USER;
GRANT SELECT ON ${YAMBA_DATABASE_DATABASE}.permission_groups TO $YAMBA_DATABASE_USER;
GRANT SELECT ON ${YAMBA_DATABASE_DATABASE}.permissions_to_permission_groups TO $YAMBA_DATABASE_USER;
GRANT SELECT ON ${YAMBA_DATABASE_DATABASE}.users_to_permissions TO $YAMBA_DATABASE_USER;
GRANT SELECT ON ${YAMBA_DATABASE_DATABASE}.users_to_permission_groups TO $YAMBA_DATABASE_USER;
GRANT SELECT ON ${YAMBA_DATABASE_DATABASE}.ts3_groups TO $YAMBA_DATABASE_USER;
GRANT SELECT ON ${YAMBA_DATABASE_DATABASE}.ts3_groups_to_permission_groups TO $YAMBA_DATABASE_USER;
FLUSH PRIVILEGES;
" $YAMBA_DATABASE_DATABASE;