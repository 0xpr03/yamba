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

/**
 * @property \App\Controller\Component\EmailComponent $Email
 */
class UsersController extends AppController
{
    public function beforeFilter(Event $event)
    {
        parent::beforeFilter($event);
        $this->Auth->allow(['add', 'index', 'logout', 'verify']);
    }

    public function initialize()
    {
        parent::initialize();
        $this->loadComponent('Email');
    }

    public function index()
    {
        return $this->redirect(['action' => 'login']);
    }

    public function add()
    {
        if ($this->isLoggedIn()) {
            return $this->redirect($this->Auth->redirectUrl());
        }

        $usersTable = TableRegistry::getTableLocator()->get('Users');
        $user = $usersTable->newEntity();
        if ($this->request->is('post')) {
            $email = $this->request->getData('email');
            $password = $this->request->getData('password');
            if (!isset($email, $password)) {
                return $this->response->withStatus(400)->withStringBody('Bad request');
            }
            $user->set('email', $email);
            $user->set('password', $password);

            $user = $this->Email->registerMail($usersTable, $user);
            if ($user) {
                $this->Auth->setUser($user);
                return $this->redirect(['action' => 'login']);
            }
        }
        $this->set('title', 'Register');
        $this->set('minlength', Configure::read('password_minlength'));
        $this->set('user', $user);
        return null;
    }

    public function login()
    {
        if ($this->isLoggedIn()) {
            return $this->redirect($this->Auth->redirectUrl());
        }

        if ($this->request->is('post')) {
            $user = $this->Auth->identify();
            if ($user) {
                if ($this->isVerifiedUser($user)) {
                    $this->Auth->setUser($user);
                    return $this->redirect($this->Auth->redirectUrl());
                } else {
                    $this->Flash->error(__('You need to verify your email before you can login'));
                }
            } else {
                $this->Flash->error(__('Invalid username or password, try again'));
            }
        }
        $this->set('minlength', Configure::read('password_minlength'));
        $this->set('title', 'Login');
        return null;
    }

    public function logout()
    {
        return $this->redirect($this->Auth->logout());
    }

    public function verify($token)
    {
        $confirmTable = TableRegistry::getTableLocator()->get('UsersNotConfirmed');
        $confirmTable->delete($confirmTable->get($token));
        $this->Flash->success(__('Your email has been verified'));
        return $this->redirect(['action' => 'login']);
    }

    private function isLoggedIn()
    {
        return $this->request->getSession()->read('Auth.User');
    }

    private function isVerifiedUser($user)
    {
        return !Configure::read('emailVerification')
            || !TableRegistry::getTableLocator()->get('UsersNotConfirmed')->find('all', ['conditions' => ['user_id' => $user['id']]])->first();
    }
}