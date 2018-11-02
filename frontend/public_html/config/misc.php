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

return [
    'password_minlength' => env('PASSWORD_MINLENGTH', 3),
    'Websocket' => [
        'ssl' => false,
        'host' => '0.0.0.0',
        'externalHost' => env('SERVER_ADDR'),
        'port' => 81,
        'frontendPath' => [
            'ssl' => [
                'path' => '/wss/',
                'usePort' => false
            ],
            'normal' => [
                'path' => '/',
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
            'user' => env('DATABASE_USERNAME', 'root'),
            'pass' => env('ROOT_PASSWORD', '1234fuenf'),
        ],
    ],
];