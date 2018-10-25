<?php
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