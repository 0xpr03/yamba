<?php
namespace App\Mailer\Preview;

use DebugKit\Mailer\MailPreview;

class UserMailPreview extends MailPreview
{
    public function welcome()
    {
        $email = 'tristan.schoenhals@mni.thm.de';
        return $this->getMailer('User')
            ->setSubject('Yamba Email Verification')
            ->setLayout('default')
            ->setTemplate('welcome')
            ->setEmailFormat('html')
            ->setTo($email)
            ->setViewVars([
                'email' => $email,
                'token' => '109238102983',
            ]);
    }
}