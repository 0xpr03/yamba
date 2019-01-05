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
use Cake\ORM\TableRegistry;

/**
 * @property \App\Controller\Component\EmailComponent $Email
 */
class InstancesController extends AppController
{

    public function index()
    {
    }

    const INSTANCE_TYPES = array(
        'teamspeak_instances'
    );

    public function addInstance()
    {
        if (!$this->request->is('post')) {
            return $this->response->withStatus(405);
        }
        $name = $this->request->getData('name');
        $autostart = $this->request->getData('autostart');
        $instance_data = $this->request->getData('instance_data');
        if (!isset($name, $autostart, $instance_data) || !in_array($instance_data['type'], self::INSTANCE_TYPES)) {
            return $this->response->withStatus(400);
        }

        $this->_updateInstances();
        return $this->response->withStatus(200);
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
    }
}