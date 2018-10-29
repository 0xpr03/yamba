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

namespace App\Controller\Component;

use Cake\Controller\Component;
use Cake\Core\Configure;
use Cake\Mailer\MailerAwareTrait;
use Cake\ORM\TableRegistry;
use Cake\Utility\Security;

/**
 * @property \Cake\Controller\Component\FlashComponent $Flash
 */
class EmailComponent extends Component
{
    public $components = ['Flash'];
    use MailerAwareTrait;

    /**
     * @param \Cake\ORM\Table $usersTable
     * @param \Cake\Datasource\EntityInterface $user
     * @return bool|\Cake\Datasource\EntityInterface
     */
    public function registerMail(\Cake\ORM\Table $usersTable, \Cake\Datasource\EntityInterface $user)
    {
        $flashesNew = [
            'couldntSave' => __('Unable to add the user'),
            'success' => __('Successfully registered'),
        ];
        $flashesOld = [
            'couldntSave' => __('Unable to change email'),
            'success' => __('Successfully changed email'),
        ];

        $newUser = $user->isNew();
        if (!$usersTable->find('all', ['conditions' => ['email' => $user['email']]])->first()) {
            if ($usersTable->save($user)) {
                if (Configure::read('emailVerification')) {
                    $confirmedTable = TableRegistry::get('UsersNotConfirmed');
                    $confirmed = $confirmedTable->newEntity();
                    $confirmed->set('user_id', $user->get('id'));
                    $confirmed->set('confirmationToken', Security::hash($user->get('id')));
                    if ($confirmedTable->save($confirmed)) {
                        $this->getMailer('User')->send('welcome', [$user, $confirmed]);
                        $this->Flash->success(__('An activation link has been sent to ') . $user->get('email'));
                        return $user;
                    } else {
                        $this->Flash->error($newUser ? $flashesNew['couldntSave'] : $flashesOld['couldntSave']);
                        $usersTable->delete($user);
                    }
                } else {
                    $this->Flash->success($newUser ? $flashesNew['success'] : $flashesOld['success']);
                    return $user;
                }
            } else {
                $this->Flash->error($newUser ? $flashesNew['couldntSave'] : $flashesOld['couldntSave']);
            }
        } else {
            $this->Flash->error(__('This email address is assigned to another user'));
        }
        return false;
    }
}