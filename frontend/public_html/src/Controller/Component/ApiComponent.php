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

namespace App\Controller\Component;

use Cake\Controller\Component;
use Cake\Http\Client;
use Cake\ORM\TableRegistry;

class ApiComponent extends Component
{
    private $backendAddress = 'http://backend:1338';

    /**
     * @param String $url
     * @return Client\Response
     */
    public function createTitles($url)
    {
        $http = new Client();
        return $http->post($this->backendAddress . '/new/titles', json_encode(['url' => $url]));
    }

    /**
     * @param array $title_ids
     * @return Client\Response
     */
    public function deleteTitles($title_ids)
    {
        $http = new Client();
        return $http->post($this->backendAddress . '/delete/titles',
            json_encode([
                'titles' => $title_ids
            ])
        );
    }

    /**
     * @param String $title_id
     * @return Client\Response
     */
    public function deleteTitle($title_id)
    {
        $http = new Client();
        return $http->post($this->backendAddress . '/delete/titles',
            json_encode([
                'titles' => [
                    $title_id
                ]
            ])
        );
    }

    public function cancelJobs()
    {

    }

    /**
     * @return Client\Response
     */
    public function notifyInstances()
    {
        $http = new Client();
        return $http->post($this->backendAddress . '/notify/updateInstances');
    }
}