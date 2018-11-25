<?php
/**
 * Created by PhpStorm.
 * User: tony
 * Date: 25.11.18
 * Time: 01:27
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
     * @param String $playlist_id
     * @return Client\Response
     */
    public function deleteTitles($playlist_id)
    {
        $titlesToPlaylistTable = TableRegistry::getTableLocator()->get('TitlesToPlaylists');
        $http = new Client();
        return $http->post($this->backendAddress . '/delete/titles',
            json_encode([
                'titles' => array_map(
                    function ($title) {
                        return $title->title_id;
                    },
                    $titlesToPlaylistTable->find('all', [
                        'conditions' => [
                            'playlist_id' => $playlist_id
                        ]
                    ])->select('title_id')->toArray()
                )
            ])
        );
    }

    public function cancelJobs()
    {

    }
}