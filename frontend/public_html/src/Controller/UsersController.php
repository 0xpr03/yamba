<?php
/**
 * Created by PhpStorm.
 * User: tony
 * Date: 20.10.18
 * Time: 15:21
 */

namespace App\Controller;

use Cake\Event\Event;
use Cake\ORM\TableRegistry;
use Cake\Mailer\MailerAwareTrait;
use Cake\Utility\Security;

class UsersController extends AppController
{
    public function beforeFilter(Event $event)
    {
        parent::beforeFilter($event);
        $this->Auth->allow(['add', 'logout', 'verify']);
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
                    $confirmedTable = TableRegistry::get('UsersNotConfirmed');
                    $confirmed = $confirmedTable->newEntity();
                    $confirmed->set('user_id', $user->get('id'));
                    $confirmed->set('confirmationToken', Security::hash($user->get('id')));
                    if($confirmedTable->save($confirmed)) {
                        $this->getMailer('User')->send('welcome', [$user, $confirmed]);
                        $this->Flash->success(__('An activation link has been sent to ') . $user->get('email'));
                        return $this->redirect(['action' => 'login']);
                    } else {
                        $this->Flash->error(__('Unable to add the user.'));
                        $usersTable->delete($user);
                    }
                } else {
                    $this->Flash->error(__('Unable to add the user.'));
                }
            } catch (\PDOException $e) {
                $this->Flash->error(__('This email address is assigned to another user!'));
            }
        }
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
                $this->Auth->setUser($user);
                return $this->redirect($this->Auth->redirectUrl());
            }
            $this->Flash->error(__('Invalid username or password, try again'));
        }
    }

    public function logout()
    {
        return $this->redirect($this->Auth->logout());
    }

    public function isLoggedIn() {
        return $this->request->getSession()->read('Auth.User');
    }

    public function verify($token) {
        $confirmTable = TableRegistry::get('UsersNotConfirmed');
        $confirmTable->delete($confirmTable->get($token));
        $this->Flash->success(__('Your email has been verified!'));
        return $this->redirect(['action' => 'login']);
    }
}