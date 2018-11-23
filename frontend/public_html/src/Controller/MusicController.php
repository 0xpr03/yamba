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
        $this->Auth->allow(['addSongs']);
        $this->Security->setConfig('unlockedActions', ['addSongs']);
    }

    public function index()
    {

    }

    public function addSongs()
    {
        $errorFunc = function ($message, $type = null, $class = null) {
            $this->_updatePlaylists($type, $class);
            return $this->response->withStatus(500)->withStringBody(__('An error occurred during addSongs: ') . __($message));
        };
        if (env('SERVER_PORT') != 82) {
            return $this->response->withStatus(403)->withStringBody('Forbidden');
        }
        $this->log($this->request->getData());
        $token = $this->request->getData('request_id');
        $song_ids = $this->request->getData('song_ids');
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
        $addSongTable = TableRegistry::getTableLocator()->get('add_songs_jobs');
        $addSong = $addSongTable->get($token);
        foreach ($song_ids as $song_id) {
            $titlesToPlaylist = $titlesToPlaylistTable->newEntity();
            $titlesToPlaylist->set('title_id', $song_id);
            $titlesToPlaylist->set('playlist_id', $addSong->get('playlist_id'));
            if (!$titlesToPlaylistTable->save($titlesToPlaylist)) {
                return $errorFunc('Error saving title_to_playlist');
            }
        }

        $addSongTable->delete($addSong);

        $playlistTable = TableRegistry::getTableLocator()->get('Playlists');
        $playlist = $playlistTable->find('all', ['conditions' => ['id' => $addSong->get('playlist_id')]])->first();
        $this->_updatePlaylists('success', 'Your playlist: "' . $playlist->get('name') . '" has been fully loaded!');
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
                    $addSongTable = TableRegistry::getTableLocator()->get('add_songs_jobs');
                    $addSong = $addSongTable->newEntity();
                    $addSong->set('backend_token', $response->json['request id']);
                    $addSong->set('playlist_id', $playlist->get('id'));
                    $addSong->set('user_id', $this->Auth->user('id'));
                    if ($addSongTable->save($addSong)) {
                        $this->_updatePlaylists();
                        return $this->response->withStatus(200)->withStringBody('Your playlist is now in processing. You will be notified once it is fully loaded');
                    } else {
                        return $errorFunc('Unable to create addSongJob');
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