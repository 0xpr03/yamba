<?php
/**
 *  This file is part of yamba.
 *
 *  yamba is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  yamba is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with yamba.  If not, see <https://www.gnu.org/licenses/>.
 */

$port = parse_url(env('HTTP_HOST'), PHP_URL_PORT);
if (empty($port)) $port = 81; //This needs to be the port specified in the nginx websocket proxy pass

return [
    'password_minlength' => env('YAMBA_PASSWORD_MINLENGTH', 3),
    'Websocket' => [
        'ssl' => false,
        'host' => '127.0.0.1',
        'externalHost' => parse_url(env('HTTP_HOST'), PHP_URL_HOST),
        'port' => $port,
        'frontendPath' => [
            'ssl' => [
                'path' => '/wss/',
                'usePort' => false
            ],
            'normal' => [
                'path' => '/socket/',
                'usePort' => true
            ]
        ],
        'sessionCookieName' => 'cws',
        'Queue' => [
            'name' => 'websocket',
            'loopInterval' => 0.1,
        ]
    ],
    'Queuesadilla' => [
        'default' => [
            'engine' => 'josegonzalez\Queuesadilla\Engine\MysqlEngine',
            'database' => 'queuesadilla',
            'host' => 'database',
            'user' => env('YAMBA_DATABASE_USERNAME'),
            'pass' => env('MYSQL_ROOT_PASSWORD'),
        ],
    ],
];
