<?php
/**
 * Created by PhpStorm.
 * User: tony
 * Date: 26.10.18
 * Time: 17:04
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