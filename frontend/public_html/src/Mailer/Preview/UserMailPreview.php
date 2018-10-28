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

namespace App\Mailer\Preview;

use Cake\ORM\TableRegistry;
use DebugKit\Mailer\MailPreview;

class UserMailPreview extends MailPreview
{
    public function welcome()
    {
        $user = TableRegistry::get('Users')->find()->first();
        return $this->getMailer('User')
            ->welcome($user, TableRegistry::get('UsersNotConfirmed')->find('all', ['conditions' => ['user_id' => $user['id']]])->first());
    }
}