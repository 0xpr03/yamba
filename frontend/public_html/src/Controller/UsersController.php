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

class UsersController extends AppController
{
    public function beforeFilter(Event $event)
    {
        parent::beforeFilter($event);
        $this->Auth->allow(['add', 'logout']);
    }

    public function index()
    {
        return $this->redirect(['action' => 'login']);
    }

    public function view($id)
    {
        $usersTable = TableRegistry::get('Users');
        $user = $usersTable->get($id);
        $this->set(compact('user'));
    }

    public function add()
    {
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
            $created = date("Y-m-d H:i:s");
            $user->set('created', $created);
            $user->set('modified', $created);
            try {
                if ($usersTable->save($user)) {
                    $this->Flash->success(__('The user has been saved.'));
                    return $this->redirect(['action' => 'login']);
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
}