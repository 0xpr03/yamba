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
use Cake\Mailer\MailerAwareTrait;
use Cake\Utility\Security;

class UsersController extends AppController
{
    public function beforeFilter(Event $event)
    {
        parent::beforeFilter($event);
        $this->Auth->allow(['add', 'index', 'logout', 'verify']);
    }

    public function index()
    {
        return $this->redirect(['action' => 'login']);
    }

    use MailerAwareTrait;

    public function add()
    {
        if ($this->isLoggedIn()) {
            return $this->redirect($this->Auth->redirectUrl());
        }

        $usersTable = TableRegistry::get('Users');
        $user = $usersTable->newEntity();
        if ($this->request->is('post')) {
            $email = $this->request->getData('email');
            $password = $this->request->getData('password');
            if (!isset($email, $password)) {
                return $this->response->withStatus(400)->withStringBody('Bad request');
            }
            $user->set('email', $email);
            $user->set('password', $password);
            try {
                if ($usersTable->save($user)) {
                    $this->sendEmail($usersTable, $user);
                } else {
                    $this->Flash->error(__('Unable to add the user'));
                }
            } catch (\PDOException $e) {
                $this->Flash->error(__('This email address is assigned to another user'));
            }
        }
        $this->set('minlength', Configure::read('password_minlength'));
        $this->set('user', $user);
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
                    return;
                }
            }
            $this->Flash->error(__('Invalid username or password, try again'));
        }
        $this->set('minlength', Configure::read('password_minlength'));
    }

    public function logout()
    {
        return $this->redirect($this->Auth->logout());
    }

    public function verify($token) {
        $confirmTable = TableRegistry::get('UsersNotConfirmed');
        $confirmTable->delete($confirmTable->get($token));
        $this->Flash->success(__('Your email has been verified'));
        return $this->redirect(['action' => 'login']);
    }

    private function isLoggedIn() {
        return $this->request->getSession()->read('Auth.User');
    }

    private function sendEmail(\Cake\ORM\Table $usersTable, \Cake\Datasource\EntityInterface $user) {
        if (Configure::read('emailVerification')) {
            $confirmedTable = TableRegistry::get('UsersNotConfirmed');
            $confirmed = $confirmedTable->newEntity();
            $confirmed->set('user_id', $user->get('id'));
            $confirmed->set('confirmationToken', Security::hash($user->get('id')));
            if($confirmedTable->save($confirmed)) {
                $this->getMailer('User')->send('welcome', [$user, $confirmed]);
                $this->Flash->success(__('An activation link has been sent to ') . $user->get('email'));
                return $this->redirect(['action' => 'login']);
            } else {
                $this->Flash->error(__('Unable to add the user'));
                $usersTable->delete($user);
            }
        } else {
            $this->Auth->setUser($user);
            return $this->redirect(['action' => 'login']);
        }

    }

    private function isVerifiedUser($user) {
        return !Configure::read('emailVerification')
                || !TableRegistry::get('UsersNotConfirmed')->find('all', ['conditions' => ['user_id' => $user['id']]])->first();
    }
}