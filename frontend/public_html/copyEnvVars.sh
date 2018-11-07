#!/bin/bash
# This file is part of yamba which is released under <GPL3>. See file LICENSE or go to https://www.gnu.org/licenses/gpl.html for full license details.

echo "clear_env = no" >> /etc/php/7.2/fpm/pool.d/www.conf;
while read p;
do
    val=$(echo $p | sed -e 's/.*=//g');
    [[ $p == "YAMBA"* ]] || [[ $p == "MYSQL"* ]] && v=${p:6} && echo env[${v/=/] = \'}\' >> /etc/php/7.2/fpm/pool.d/www.conf;
    [[ $p == "YAMBA_DATABASE_USERNAME"* ]] && sed -i -e "s/<root>/${val}/g" ./config/misc.php;
    [[ $p == "MYSQL_ROOT_PASSWORD"* ]] && sed -i -e "s/<root-pw>/${val}/g" ./config/misc.php;
done < <(printenv);
exit 0;