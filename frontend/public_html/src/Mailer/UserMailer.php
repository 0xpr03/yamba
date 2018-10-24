<?php
/**
 * Created by PhpStorm.
 * User: tony
 * Date: 21.10.18
 * Time: 15:07
 */

namespace App\Mailer;

use Cake\Mailer\Mailer;

class UserMailer extends Mailer
{
    public function welcome($user, $confirmed)
    {
        $this
            ->setSubject('Yamba Email Verification')
            ->setTransport('mailjet')
            ->setLayout('default')
            ->setTemplate('welcome')
            ->setEmailFormat('html')
            ->setTo($user['email'])
            ->setViewVars([
                'email' => $user['email'],
                'token' => $confirmed['confirmationToken']
            ]);
    }

    public function resetPassword($user)
    {
        $this
            ->to($user->email)
            ->subject('Reset password')
            ->set(['token' => $user->token]);
    }
}