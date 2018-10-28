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

use Cake\Core\Configure;
use Cake\Event\Event;
use Cake\ORM\TableRegistry;

class AccountsController extends AppController
{
    public function beforeFilter(Event $event)
    {
        parent::beforeFilter($event);
    }

    public function index() {
        return $this->redirect(['action' => 'settings']);
    }

    public function settings() {
        $usersTable = TableRegistry::get('Users');
        $user = $usersTable->newEntity();
        $this->set('minlength', Configure::read('password_minlength'));
        $this->set('user', $user);
    }

    public function changePassword() {
        if ($this->request->is('post')) {
            $old_password = $this->request->getData('password');
            $new_password = $this->request->getData('new_password');
            $new_password_repeat = $this->request->getData('new_password_repeat');
            $minlength = Configure::read('password_minlength');
            if (!isset($old_password, $new_password, $new_password_repeat)
                || mb_strlen($old_password) < $minlength
                || mb_strlen($new_password) < $minlength
                || mb_strlen($new_password_repeat) < $minlength) {
                return $this->response->withStatus(400)->withStringBody('Bad request');
            }
            $this->request = $this->request->withData('email', $this->Auth->user('email'));
            $user = $this->Auth->identify();
            if ($user) {
                if ($old_password !== $new_password) {
                    if ($new_password === $new_password_repeat) {
                        $usersTable = TableRegistry::get('Users');
                        $user = $usersTable->get($user['id']);
                        $user->set('password', $new_password);
                        if ($usersTable->save($user)) {
                            $this->Auth->setUser($user);
                            $this->Flash->success(__('Password updated'));
                        } else {
                            $this->Flash->error(__('Unable to update password'));
                        }
                    } else {
                        $this->Flash->error(__('New Passwords don\'t match'));
                    }
                } else {
                    $this->Flash->error(__('Old password may not be equivalent to new password'));
                }
            } else {
                $this->Flash->error(__('Wrong password'));
            }
            return $this->redirect(['action' => 'settings']);
        }
    }
}