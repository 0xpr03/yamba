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


use Cake\ORM\TableRegistry;

class MusicController extends AppController
{
    public function index()
    {

    }

    public function addPlaylist()
    {
        $name = $this->request->getQuery('name');
        if (!isset($name) || mb_strlen($name) < 1) {
            return $this->response->withStatus(400)->withStringBody('Bad request');
        }
        $playlistTable = TableRegistry::getTableLocator()->get('Playlists');
        $playlist = $playlistTable->newEntity();
        $playlist->set('name', $name);
        $playlistTable->save($playlist);
        return $this->getPlaylists();
    }

    public function getPlaylists()
    {
        $playlistTable = TableRegistry::getTableLocator()->get('Playlists');
        return $this->response->withType('json')->withStringBody(json_encode($playlistTable->find('all')->contain(['songs_to_playlists'])->orderDesc('created')));
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
        return $this->getPlaylists();
    }
}