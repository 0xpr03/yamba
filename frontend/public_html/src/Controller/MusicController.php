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


use Cake\Http\Client;
use Cake\ORM\TableRegistry;
use Websocket\Lib\Websocket;

class MusicController extends AppController
{
    public function index()
    {

    }

    public function addSongs()
    {
        $song_ids = json_decode($this->request->getData('song_ids'));
        $token = $this->request->getData('token');
        if (!isset($song_ids, $token)) {
            return $this->response->withStatus(400)->withStringBody('Bad request');
        }

        $songsToPlaylistTable = TableRegistry::getTableLocator()->get('songs_to_playlists');
        $addSongTable = TableRegistry::getTableLocator()->get('add_songs_jobs');
        $addSong = $addSongTable->get($token);
        foreach ($song_ids as $song_id) {
            $songsToPlaylist = $songsToPlaylistTable->newEntity();
            $songsToPlaylist->set('song_id', $song_id);
            $songsToPlaylist->set('playlist_id', $addSong->get('playlist_id'));
            $songsToPlaylistTable->save($songsToPlaylist);
        }

        $addSongTable->delete($addSong);
        return $this->response->withStatus(200)->withStringBody('OK');
    }

    public function addPlaylist()
    {
        $this->autoRender = false;
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
                $response = $http->post('backend/new/playlist', json_encode(['url' => $url]));
                if ($response->getStatusCode() === 202) {
                    $addSongTable = TableRegistry::getTableLocator()->get('add_songs_jobs');
                    $addSong = $addSongTable->newEntity();
                    $addSong->set('backend_token', $response->body('json_decode'));
                    $addSong->set('playlist_id', $playlist->get('id'));
                    $addSong->set('user_id', $this->getRequest()->getSession()->read('User.id'));
                    if ($addSongTable->save($addSong)) {
                        $this->Flash->success(__('Your playlist is now in processing. You will be notified once it is fully loaded'));
                    }
                } else {
                    $this->Flash->error(__('An error occurred while fetching the URL data'));
                }
            }
        } else {
            $this->Flash->error(__('Could not save the playlist'));
        }

        $this->_updatePlaylists();
        return null;
    }

    private function _playlistsJson()
    {
        $playlistTable = TableRegistry::getTableLocator()->get('Playlists');
        return json_encode($playlistTable->find('all')->contain(['songs_to_playlists'])->orderDesc('created'));
    }

    public function getPlaylists()
    {
        return $this->response->withType('json')->withStringBody($this->_playlistsJson());
    }

    public function deletePlaylist()
    {
        $this->autoRender = false;
        $id = $this->request->getQuery('id');
        if (!isset($id) || mb_strlen($id) !== 36) {
            return $this->response->withStatus(400)->withStringBody('Bad request');
        }
        $playlistTable = TableRegistry::getTableLocator()->get('Playlists');
        $playlist = $playlistTable->get($id);
        $playlistTable->delete($playlist);
        $this->_updatePlaylists();
        return null;
    }

    private function _updatePlaylists()
    {
        Websocket::publishEvent('playlistsUpdated', ['json' => $this->_playlistsJson()]);
    }
}