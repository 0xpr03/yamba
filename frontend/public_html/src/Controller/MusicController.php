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
        if (env('SERVER_PORT') != 82) {
            return $this->response->withStatus(403)->withStringBody('Forbidden');
        }
        $token = $this->request->getData('token');
        $song_ids = $this->request->getData('song_ids');
        if (!isset($song_ids, $token)) {
            return $this->response->withStatus(400)->withStringBody('Bad request');
        }

        $songsToPlaylistTable = TableRegistry::getTableLocator()->get('titles_to_playlists');
        $addSongTable = TableRegistry::getTableLocator()->get('add_songs_jobs');
        $addSong = $addSongTable->get($token);
        foreach ($song_ids as $song_id) {
            $songsToPlaylist = $songsToPlaylistTable->newEntity();
            $songsToPlaylist->set('song_id', $song_id);
            $songsToPlaylist->set('playlist_id', $addSong->get('playlist_id'));
            if (!$songsToPlaylistTable->save($songsToPlaylist)) {
                return $this->response->withStatus(500)->withStringBody('Interal Server Error');
            }
        }

        $addSongTable->delete($addSong);
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

    private function _updatePlaylists()
    {
        Websocket::publishEvent('playlistsUpdated', ['json' => $this->_playlistsJson()]);
    }
}