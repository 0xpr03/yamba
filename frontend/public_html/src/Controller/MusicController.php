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
use Cake\Datasource\Exception\RecordNotFoundException;
use Cake\Event\Event;
use Cake\ORM\TableRegistry;
use Websocket\Lib\Websocket;

/**
 * @property \App\Controller\Component\ApiComponent $Api
 */
class MusicController extends AppController
{
    public function beforeFilter(Event $event)
    {
        parent::beforeFilter($event);
        $this->Auth->allow(['addTitles']);
        $this->Security->setConfig('unlockedActions', ['addTitles']);
    }

    public function initialize()
    {
        parent::initialize();
        $this->loadComponent('Api');
    }

    public function index()
    {
        $this->FrontendBridge->setJson(
            'userID',
            $this->Auth->user('id')
        );
    }

    public function addTitles()
    {
        if (env('SERVER_PORT') != 82) {
            return $this->response->withStatus(403);
        }
        $token = $this->request->getData('request_id');
        $title_ids = $this->request->getData('song_ids');
        $code = $this->request->getData('error_code');
        $message = $this->request->getData('message');

        if (!isset($token, $code)) {
            return $this->response->withStatus(400);
        }

        $addTitleTable = TableRegistry::getTableLocator()->get('AddTitlesJobs');
        try {
            // Retrieve job entry and delete from database
            $addTitleTable->delete($addTitle = $addTitleTable->get($token));
        } catch (RecordNotFoundException $e) {
            // If there is no job (likely because a user has deleted it), we can ignore the callback
            return $this->response->withStatus(200);
        }
        $userID = $addTitle->get('user_id');

        switch ($code) {
            case 0:
                $titlesToPlaylistTable = TableRegistry::getTableLocator()->get('TitlesToPlaylists');
                $playlistTable = TableRegistry::getTableLocator()->get('Playlists');

                $playlistID = $addTitle->get('playlist_id');
                $playlistName = $playlistTable->find('all', ['conditions' => ['id' => $playlistID]])->select('name')->first()->get('name');

                $incompleteTitles = false;
                $attemptedTitleCount = 0;
                foreach ($title_ids as $title_id) {
                    $titlesToPlaylist = $titlesToPlaylistTable->newEntity();
                    $titlesToPlaylist->set('title_id', $title_id);
                    $titlesToPlaylist->set('playlist_id', $playlistID);
                    $attemptedTitleCount++;
                    if (!$titlesToPlaylistTable->save($titlesToPlaylist)) {
                        $this->log("Error saving titles_to_playlists");
                        $incompleteTitles = true;
                    }
                }

                $titleCount = $titlesToPlaylistTable->find('all', ['conditions' => ['playlist_id' => $playlistID]])->count();
                if ($incompleteTitles) {
                    $type = 'warning';
                    $message = $titleCount . ' out of ' . $attemptedTitleCount . 'have successfully been added to "' . $playlistName . '"';
                } else {
                    $type = 'success';
                    $message = $titleCount . ' titles have been successfully loaded into "' . $playlistName . '"';
                }
                $this->_updatePlaylists();
                $this->_updateTitles($playlistID);
                $this->_flash($type, $message, $userID);
                return $this->response->withStatus(200);
            default:
                $this->_flash('alert', $message, $userID);
                return $this->response->withStatus(500);
        }
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
        $playlist = $playlistTable->save($playlist);
        $status = 500;
        try {
            if ($playlist) {
                $url = $this->request->getQuery('url');
                if (isset($url) && mb_strlen($url) > 0) {
                    $response = $this->Api->createTitles($url);
                    if ($response->getStatusCode() === 202) {
                        $addTitleTable = TableRegistry::getTableLocator()->get('AddTitlesJobs');
                        $addTitle = $addTitleTable->newEntity();
                        $addTitle->set('backend_token', $response->getJson()['request_id']);
                        $addTitle->set('playlist_id', $playlist->get('id'));
                        $addTitle->set('user_id', $this->Auth->user('id'));
                        if ($addTitleTable->save($addTitle)) {
                            $status = 200;
                            $message = 'Your playlist is now in processing. You will be notified once it is fully loaded';
                        } else {
                            $message = 'Database Error';
                        }
                    } else {
                        $message = 'Could not resolve URL';
                    }
                } else {
                    $status = 200;
                    $message = 'OK';
                }
            } else {
                $message = 'Could not save the playlist';
            }
        } catch (Exception $e) {
            $message = 'Unable to connect to backend';
        }

        $this->_updatePlaylists();
        return $this->response->withStatus($status)->withStringBody($message);
    }

    private function _playlistsJson()
    {
        $playlistTable = TableRegistry::getTableLocator()->get('Playlists');
        $query = $playlistTable->find();
        return json_encode($query->select([
            'Playlists.id',
            'Playlists.name',
            'titles' => $query->func()->count('TitlesToPlaylists.title_id'),
            'hasToken' => $query->newExpr()
                ->addCase(
                    [$query->newExpr()->isNotNull('AddTitlesJobs.backend_token')],
                    [true, false],
                    ['boolean', 'boolean']
                )

        ])
            ->leftJoinWith('TitlesToPlaylists')
            ->contain('AddTitlesJobs')
            ->group('Playlists.id')
            ->orderDesc('created'));
    }

    public function getPlaylists()
    {
        return $this->response->withType('json')->withStringBody($this->_playlistsJson());
    }

    public function deletePlaylist()
    {
        $id = $this->request->getQuery('id');
        if (!isset($id) || mb_strlen($id) !== 36) {
            return $this->response->withStatus(400);
        }
        $playlistTable = TableRegistry::getTableLocator()->get('Playlists');
        $playlist = $playlistTable->get($id);

        $titlesToPlaylistTable = TableRegistry::getTableLocator()->get('TitlesToPlaylists');
        if ($titlesToPlaylistTable->find()->where(['playlist_id' => $id])->count()) {
            try {
                $this->Api->deleteTitles($playlist->get('id'));
            } catch (Exception $e) {
                return $this->response->withStatus(500)->withStringBody('Unable to connect to backend');
            }
        }

        $playlistTable->delete($playlist);
        $this->_updatePlaylists();
        return $this->response->withStatus(200);
    }

    public function deleteTitle($playlist_id, $title_id)
    {
        $titlesToPlaylistTable = TableRegistry::getTableLocator()->get('TitlesToPlaylists');
        $titlesToPlaylistTable->delete($titlesToPlaylistTable->get([$title_id, $playlist_id]));
        if (!$titlesToPlaylistTable->find()->where(['title_id' => $title_id])->count()) {
            try {
                $this->Api->deleteTitle($title_id);
            } catch (Exception $e) {
                return $this->response->withStatus(500)->withStringBody('Unable to connect to backend');
            }
        }
        $this->_updatePlaylists();
        $this->_updateTitles($playlist_id);
        return $this->response->withStatus(200);
    }

    private function _updateTitles($playlist_id)
    {
        Websocket::publishEvent('titlesUpdated', ['json' => $this->_titlesJson($playlist_id), 'playlist' => $playlist_id]);
    }

    private function _titlesJson($playlist_id)
    {
        $titlesTable = TableRegistry::getTableLocator()->get('Titles');
        return json_encode($titlesTable->find()->leftJoinWith('TitlesToPlaylists')->where(['TitlesToPlaylists.playlist_id' => $playlist_id]));
    }

    public function getTitles($playlist_id)
    {
        return $this->response->withType('json')->withStringBody($this->_titlesJson($playlist_id));
    }

    private function _updatePlaylists()
    {
        Websocket::publishEvent('playlistsUpdated', ['json' => $this->_playlistsJson()]);
    }

    private function _flash($type = null, $message = null, $userID = null)
    {
        if ($userID == null) {
            $userID = $this->Auth->user('id');
        }
        Websocket::publishEvent('flash', ['type' => $type, 'message' => $message, 'userID' => $userID]);
    }
}