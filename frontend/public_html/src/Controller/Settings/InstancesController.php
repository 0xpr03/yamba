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

namespace App\Controller\Settings;

use App\Controller\AppController;
use Cake\Core\Exception\Exception;
use Cake\ORM\TableRegistry;
use Websocket\Lib\Websocket;

/**
 * @property \App\Controller\Component\EmailComponent $Email
 * @property \App\Controller\Component\ApiComponent $Api
 */
class InstancesController extends AppController
{
    public function initialize()
    {
        parent::initialize();
        $this->loadComponent('Api');
    }

    public function index()
    {
        return $this->redirect(['action' => 'updateInstance']);
    }

    public function addInstance()
    {
        if ($this->request->is('post')) {
            $name = $this->request->getData('name');
            $autostart = $this->request->getData('autostart');
            $type = $this->request->getData('type');
            if (!isset($name, $autostart, $type)) {
                return $this->response->withStatus(400);
            }
            switch ($type) {
                case 'teamspeak_instances':
                    $host = $this->request->getData('teamspeak-host');
                    $identity = $this->request->getData('teamspeak-identity');
                    $password = $this->request->getData('teamspeak-password');
                    if (!isset($host, $identity, $password)) {
                        return $this->response->withStatus(400);
                    }
                    $instanceTable = TableRegistry::getTableLocator()->get('Instances');
                    $instance = $instanceTable->newEntity([
                        'name' => $name,
                        'type' => $type,
                        'autostart' => $autostart == '0' ? false : true,
                        'teamspeak_instance' => [
                            'host' => $host,
                            'identity' => $identity,
                            'password' => $password
                        ]
                    ]);
                    if ($instanceTable->save($instance)) {
                        $this->_updateInstances();
                        return $this->redirect(['action' => 'index']);
                    } else {
                        return $this->response->withStatus(500);
                    }
                    break;
                default:
                    return $this->response->withStatus(400);
            }
        }
        $this->set('submitText', 'Add Instance');
        return null;
    }

    public function updateInstance()
    {
        if ($this->request->is('post')) {
            $name = $this->request->getData('name');
            $autostart = $this->request->getData('autostart');
            $type = $this->request->getData('type');
            $id = $this->request->getData('id');
            if (!isset($name, $autostart, $type, $id)) {
                return $this->response->withStatus(400);
            }
            switch ($type) {
                case 'teamspeak_instances':
                    $host = $this->request->getData('teamspeak-host');
                    $identity = $this->request->getData('teamspeak-identity');
                    $password = $this->request->getData('teamspeak-password');
                    if (!isset($host)) {
                        return $this->response->withStatus(400);
                    }
                    $instanceTable = TableRegistry::getTableLocator()->get('Instances');
                    $instance = $instanceTable->get($id, ['contain' => 'TeamspeakInstances']);
                    $instance->set([
                        'name' => $name,
                        'type' => $type,
                        'autostart' => $autostart == '0' ? false : true,
                        'teamspeak_instance' => $instance->get('teamspeak_instance')->set([
                            'host' => $host,
                            'identity' => $identity,
                            'password' => $password
                        ])
                    ]);
                    if ($instanceTable->save($instance)) {
                        $this->_updateInstances();
                        return $this->redirect(['action' => 'index']);
                    } else {
                        return $this->response->withStatus(500);
                    }
                    break;
                default:
                    return $this->response->withStatus(400);
            }
        }
        $this->set('submitText', 'Update Instance');
        return null;
    }

    private function _instancesJson()
    {
        $instanceTable = TableRegistry::getTableLocator()->get('Instances');
        return json_encode($instanceTable->find()->contain('TeamspeakInstances'));
    }

    public function getInstances()
    {
        return $this->response->withType('json')->withStringBody($this->_instancesJson());
    }

    private function _updateInstances()
    {
        Websocket::publishEvent('instancesUpdated', ['json' => $this->_instancesJson()]);
        try {
            $this->Api->notifyInstances();
        } catch (Exception $e) {
            return $this->response->withStatus(500)->withStringBody('Unable to connect to backend');
        }
    }
}