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

namespace App\Controller;


use Cake\Core\Exception\Exception;
use Cake\Event\Event;
use Cake\Http\Client;
use Cake\ORM\TableRegistry;
use Websocket\Lib\Websocket;

class MusicController extends AppController
{

    public function beforeFilter(Event $event)
    {
        parent::beforeFilter($event);
        $this->Auth->allow(['addTitles']);
        $this->Security->setConfig('unlockedActions', ['addTitles']);
    }

    public function index()
    {

    }

    public function addTitles()
    {
        $errorFunc = function ($message, $type = null, $class = null) {
            $this->_updatePlaylists($type, $class);
            return $this->response->withStatus(500)->withStringBody(__('An error occurred during addTitles: ') . __($message));
        };
        if (env('SERVER_PORT') != 82) {
            return $this->response->withStatus(403)->withStringBody('Forbidden');
        }
        $this->log($this->request->getData());
        $token = $this->request->getData('request_id');
        $title_ids = $this->request->getData('song_ids');
        $code = $this->request->getData('error_code');
        $message = $this->request->getData('message');

        if (!isset($token, $code)) {
            return $this->response->withStatus(400)->withStringBody('Bad request');
        }

        switch ($code) {
            case 0:
                break;
            default:
                return $errorFunc('Unknown code: ' . $code, 'alert', $message);
        }

        $titlesToPlaylistTable = TableRegistry::getTableLocator()->get('titles_to_playlists');
        $addTitleTable = TableRegistry::getTableLocator()->get('add_titles_jobs');
        $addTitle = $addTitleTable->get($token);
        foreach ($title_ids as $title_id) {
            $titlesToPlaylist = $titlesToPlaylistTable->newEntity();
            $titlesToPlaylist->set('title_id', $title_id);
            $titlesToPlaylist->set('playlist_id', $addTitle->get('playlist_id'));
            if (!$titlesToPlaylistTable->save($titlesToPlaylist)) {
                return $errorFunc('Error saving title_to_playlist');
            }
        }

        $addTitleTable->delete($addTitle);

        $playlistTable = TableRegistry::getTableLocator()->get('Playlists');
        $playlist = $playlistTable->find('all', ['conditions' => ['id' => $addTitle->get('playlist_id')]])->first();
        $titleCount = $titlesToPlaylistTable->find('all', ['conditions' => ['playlist_id' => $addTitle->get('playlist_id')]])->count();
        $this->_updatePlaylists('success', $titleCount . ' titles have been successfully loaded into "' . $playlist->get('name') . '"');
        return $this->response->withStatus(200)->withStringBody('OK');
    }

    public function addPlaylist()
    {
        $errorFunc = function ($message) {
            $this->_updatePlaylists('alert', __('An error occurred during playlist creation: ') . __($message));
            return $this->response->withStatus(500)->withStringBody('Interal Server Error');
        };
        $name = $this->request->getQuery('name');
        if (!isset($name) || mb_strlen($name) < 1) {
            return $this->response->withStatus(400)->withStringBody('Bad request');
        }

        $playlistTable = TableRegistry::getTableLocator()->get('Playlists');
        $playlist = $playlistTable->newEntity();
        $playlist->set('name', $name);
        $playlist = $playlistTable->save($playlist);
        if ($playlist) {
            $url = $this->request->getQuery('url');
            if (isset($url) && mb_strlen($url) > 0) {
                $http = new Client();
                try {
                    $response = $http->post('http://backend:1338/new/playlist', json_encode(['url' => $url]));
                } catch (Exception $e) {
                    return $errorFunc('Unable to connect to backend');
                }

                if ($response->getStatusCode() === 202) {
                    $addTitleTable = TableRegistry::getTableLocator()->get('add_titles_jobs');
                    $addTitle = $addTitleTable->newEntity();
                    $addTitle->set('backend_token', $response->json['request id']);
                    $addTitle->set('playlist_id', $playlist->get('id'));
                    $addTitle->set('user_id', $this->Auth->user('id'));
                    if ($addTitleTable->save($addTitle)) {
                        $this->_updatePlaylists();
                        return $this->response->withStatus(200)->withStringBody('Your playlist is now in processing. You will be notified once it is fully loaded');
                    } else {
                        return $errorFunc('Unable to create addTitleJob');
                    }
                } else {
                    return $errorFunc('Could not resolve URL');
                }
            } else {
                $this->_updatePlaylists();
                return $this->response->withStatus(200)->withStringBody('OK');
            }
        } else {
            return $errorFunc('Could not save the playlist');
        }
    }

    private function _playlistsJson()
    {
        $playlistTable = TableRegistry::getTableLocator()->get('Playlists');
        return json_encode($playlistTable->find('all')->select(['Playlists.id', 'Playlists.name'])->contain(['titles_to_playlists'])->orderDesc('created'));
    }

    public function getPlaylists()
    {
        return $this->response->withType('json')->withStringBody($this->_playlistsJson());
    }

    public function deletePlaylist()
    {
        $id = $this->request->getQuery('id');
        if (!isset($id) || mb_strlen($id) !== 36) {
            return $this->response->withStatus(400)->withStringBody('Bad request');
        }
        $playlistTable = TableRegistry::getTableLocator()->get('Playlists');
        $playlist = $playlistTable->get($id);
        $playlistTable->delete($playlist);
        $this->_updatePlaylists();
        return $this->response->withStatus(200)->withStringBody('OK');
    }

    private function _updatePlaylists($type = null, $message = null)
    {
        Websocket::publishEvent('playlistsUpdated', ['json' => $this->_playlistsJson(), 'type' => $type, 'message' => $message]);
    }
}