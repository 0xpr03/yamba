#!/bin/bash
echo "clear_env = no" >> /etc/php/7.0/fpm/pool.d/www.conf;
while read p;
do
    [[ $p == "YAMBA"* ]] && p=${p:6} && echo env[${p/=/] = } >> /etc/php/7.0/fpm/pool.d/www.conf;
done < <(printenv);
exit 0;