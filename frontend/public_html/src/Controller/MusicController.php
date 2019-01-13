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
use Cake\Database\Expression\QueryExpression;
use Cake\Datasource\Exception\RecordNotFoundException;
use Cake\Event\Event;
use Cake\ORM\Query;
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
        if (!$this->request->is('post')) {
            return $this->response->withStatus(405);
        }
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
                    $titlesToPlaylist = $titlesToPlaylistTable->newEntity([
                        'title_id' => $title_id,
                        'playlist_id' => $playlistID
                    ]);
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
                $this->_updatePlaylists();
                $this->_flash('alert', $message, $userID);
                return $this->response->withStatus(500);
        }
    }

    public function addTitle()
    {
        if (!$this->request->is('post')) {
            return $this->response->withStatus(405);
        }
        $url = $this->request->getData('url');
        if (!isset($url)) {
            return $this->response->withStatus(400);
        }
    }

    public function addPlaylist()
    {
        if (!$this->request->is('post')) {
            return $this->response->withStatus(405);
        }
        $name = $this->request->getData('name');
        if (!isset($name) || mb_strlen($name) < 1 || mb_strlen($name) > 50) {
            return $this->response->withStatus(400);
        }

        $playlistTable = TableRegistry::getTableLocator()->get('Playlists');
        $playlist = $playlistTable->newEntity();
        $playlist->set('name', $name);
        $playlist = $playlistTable->save($playlist);
        $status = 500;
        try {
            if ($playlist) {
                $url = $this->request->getData('url');
                if (isset($url) && mb_strlen($url) > 0) {
                    $response = $this->Api->createTitles($url);
                    if ($response->getStatusCode() === 202) {
                        $addTitleTable = TableRegistry::getTableLocator()->get('AddTitlesJobs');
                        $addTitle = $addTitleTable->newEntity();
                        $addTitle->set('backend_token', $response->getJson()['request_id']);
                        $addTitle->set('playlist_id', $playlist->get('id'));
                        $addTitle->set('user_id', $this->Auth->user('id'));
                        if ($addTitleTable->save($addTitle)) {
                            $status = 202;
                            $message = 'Your playlist is now in processing. You will be notified once it is fully loaded';
                        } else {
                            $message = 'Database Error';
                        }
                    } else {
                        $message = 'Could not resolve URL';
                    }
                } else {
                    $status = 201;
                    $message = 'Your playlist has been created';
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
        $res = $query->select([
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
            ->orderAsc('created')->toList();
        foreach ($res as $p) {
            $p->hasToken = (bool)$p->hasToken;
        }
        return json_encode($res);
    }

    private function _titlesJson($playlist_id)
    {
        $titlesTable = TableRegistry::getTableLocator()->get('Titles');
        return json_encode($titlesTable->find()->leftJoinWith('TitlesToPlaylists')->where(['TitlesToPlaylists.playlist_id' => $playlist_id]));
    }

    private function _queueJson($instance_id)
    {
        $queuesTable = TableRegistry::getTableLocator()->get('Queues');
        return json_encode($queuesTable->find()->where(['instance_id' => $instance_id])->count());
    }

    private function _queueTitlesJson($instance_id)
    {
        $queuesTable = TableRegistry::getTableLocator()->get('Queues');
        return json_encode($queuesTable->find()->contain('Titles')->where(['instance_id' => $instance_id])->orderAsc('position'));
    }

    public function getPlaylists()
    {
        return $this->response->withType('json')->withStringBody($this->_playlistsJson());
    }

    public function getTitles($playlist_id)
    {
        return $this->response->withType('json')->withStringBody($this->_titlesJson($playlist_id));
    }

    public function getQueue($instance_id)
    {
        return $this->response->withType('json')->withStringBody($this->_queueJson($instance_id));
    }

    public function getQueueTitles($instance_id)
    {
        return $this->response->withType('json')->withStringBody($this->_queueTitlesJson($instance_id));
    }

    private function _updatePlaylists()
    {
        Websocket::publishEvent('playlistsUpdated', ['json' => $this->_playlistsJson()]);
    }

    private function _updateTitles($playlist_id)
    {
        Websocket::publishEvent('titlesUpdated', ['json' => $this->_titlesJson($playlist_id), 'playlist' => $playlist_id]);
    }

    private function _updateQueue($instance_id)
    {
        Websocket::publishEvent('titlesUpdated', ['json' => $this->_queueTitlesJson($instance_id), 'playlist' => 'queue', 'count' => $this->_queueJson($instance_id)]);
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
        $titles = $titlesToPlaylistTable->find()->select('title_id')->where(['playlist_id' => $id])->toArray();

        $playlistTable->delete($playlist);
        $titles = array_map(
            function ($title) {
                return $title->title_id;
            },
            array_filter(
                $titles,
                function ($title) {
                    return !$this->titleAssociated($title->title_id);
                }
            )
        );
        if ($titles) {
            try {
                $this->Api->deleteTitles($titles);
            } catch (Exception $e) {
                return $this->response->withStatus(500)->withStringBody('Unable to connect to backend');
            }
        }

        $this->_updatePlaylists();
        return $this->response->withStatus(200);
    }

    public function deleteTitle($playlist_id, $title_id, $instance_id)
    {
        $queuesTable = TableRegistry::getTableLocator()->get('Queues');
        $titlesToPlaylistTable = TableRegistry::getTableLocator()->get('TitlesToPlaylists');

        if ($playlist_id === 'queue') {
            $queuesTable->delete($queuesTable->find()->where(['instance_id' => $instance_id, 'title_id' => $title_id])->firstOrFail());
        } else {
            $titlesToPlaylistTable->delete($titlesToPlaylistTable->find()->where(['playlist_id' => $playlist_id, 'title_id' => $title_id])->firstOrFail());
        }
        if (!$this->titleAssociated($title_id)) {
            try {
                $this->Api->deleteTitle($title_id);
            } catch (Exception $e) {
                return $this->response->withStatus(500)->withStringBody('Unable to connect to backend');
            }
        }

        $this->_updatePlaylists();
        if ($playlist_id === 'queue') {
            $this->_updateQueue($instance_id);
        } else {
            $this->_updateTitles($playlist_id);
        }
        return $this->response->withStatus(200);
    }

    private function titleAssociated($title_id)
    {
        $queuesTable = TableRegistry::getTableLocator()->get('Queues');
        $titlesToPlaylistTable = TableRegistry::getTableLocator()->get('TitlesToPlaylists');

        return $titlesToPlaylistTable->find()->where(['title_id' => $title_id])->count() ||
            $queuesTable->find()->where(['title_id' => $title_id])->count();
    }

    private function _flash($type = null, $message = null, $userID = null)
    {
        if ($userID == null) {
            $userID = $this->Auth->user('id');
        }
        Websocket::publishEvent('flash', ['type' => $type, 'message' => $message, 'userID' => $userID]);
    }
}