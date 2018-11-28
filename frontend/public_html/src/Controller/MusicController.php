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
        $errorFunc = function ($message, $userID, $type = null, $class = null) {
            $this->_updatePlaylists($type, $class, $userID);
            return $this->response->withStatus(500)->withStringBody(__('An error occurred during addTitles: ') . __($message));
        };
        if (env('SERVER_PORT') != 82) {
            return $this->response->withStatus(403)->withStringBody('Forbidden');
        }
        $token = $this->request->getData('request_id');
        $title_ids = $this->request->getData('song_ids');
        $code = $this->request->getData('error_code');
        $message = $this->request->getData('message');

        if (!isset($token, $code)) {
            return $this->response->withStatus(400)->withStringBody('Bad request');
        }

        $titlesToPlaylistTable = TableRegistry::getTableLocator()->get('TitlesToPlaylists');
        $addTitleTable = TableRegistry::getTableLocator()->get('add_titles_jobs');
        try {
            $addTitle = $addTitleTable->get($token);
        } catch (RecordNotFoundException $e) {
            return $this->response->withStatus(200)->withStringBody('OK');
        }
        $userID = $addTitle->get('user_id');

        switch ($code) {
            case 0:
                break;
            default:
                return $errorFunc('Unknown code: ' . $code, $userID, 'alert', $message);
        }
        foreach ($title_ids as $title_id) {
            $titlesToPlaylist = $titlesToPlaylistTable->newEntity();
            $titlesToPlaylist->set('title_id', $title_id);
            $titlesToPlaylist->set('playlist_id', $addTitle->get('playlist_id'));
            if (!$titlesToPlaylistTable->save($titlesToPlaylist)) {
                return $errorFunc('Error saving title_to_playlist', $userID);
            }
        }

        $addTitleTable->delete($addTitle);

        $playlistTable = TableRegistry::getTableLocator()->get('Playlists');
        $playlistName = $playlistTable->find('all', ['conditions' => ['id' => $addTitle->get('playlist_id')]])->select('name')->first()->get('name');
        $titleCount = $titlesToPlaylistTable->find('all', ['conditions' => ['playlist_id' => $addTitle->get('playlist_id')]])->count();
        $this->_updatePlaylists('success', $titleCount . ' titles have been successfully loaded into "' . $playlistName . '"', $userID);
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
                try {
                    $response = $this->Api->createTitles($url);
                } catch (Exception $e) {
                    return $errorFunc('Unable to connect to backend');
                }

                if ($response->getStatusCode() === 202) {
                    $addTitleTable = TableRegistry::getTableLocator()->get('add_titles_jobs');
                    $addTitle = $addTitleTable->newEntity();
                    $addTitle->set('backend_token', $response->json['request_id']);
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
        $query = $playlistTable->find('all');
        return json_encode($query->select([
            'Playlists.id',
            'Playlists.name',
            'titles' => $query->func()->count('TitlesToPlaylists.title_id')
        ])
            ->leftJoinWith('TitlesToPlaylists')
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
            return $this->response->withStatus(400)->withStringBody('Bad request');
        }
        $playlistTable = TableRegistry::getTableLocator()->get('Playlists');
        $playlist = $playlistTable->get($id);

        try {
            $this->Api->deleteTitles($playlist->get('id'));
        } catch (Exception $e) {
            return $this->response->withStatus(500)->withStringBody('Unable to connect to backend');
        }

        $playlistTable->delete($playlist);
        $this->_updatePlaylists();
        return $this->response->withStatus(200)->withStringBody('OK');
    }

    private function _updatePlaylists($type = null, $message = null, $userID = null)
    {
        if ($userID == null) {
            $userID = $this->Auth->user('id');
        }
        Websocket::publishEvent('playlistsUpdated', ['json' => $this->_playlistsJson(), 'type' => $type, 'message' => $message, 'userID' => $userID]);
    }
}