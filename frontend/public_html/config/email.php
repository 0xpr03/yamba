<?php
/**
 * Created by PhpStorm.
 * User: tony
 * Date: 25.10.18
 * Time: 10:47
 */
return [
    /**
     * Email configuration.
     *
     * By defining transports separately from delivery profiles you can easily
     * re-use transport configuration across multiple profiles.
     *
     * You can specify multiple configurations for production, development and
     * testing.
     *
     * Each transport needs a `className`. Valid options are as follows:
     *
     *  Mail   - Send using PHP mail function
     *  Smtp   - Send using SMTP
     *  Debug  - Do not send the email, just return the result
     *
     * You can add custom transports (or override existing transports) by adding the
     * appropriate file to src/Mailer/Transport. Transports should be named
     * 'YourTransport.php', where 'Your' is the name of the transport.
     */
    'emailVerification' => filter_var(env('EMAIL_VERIFICATION', false), FILTER_VALIDATE_BOOLEAN),

    'EmailTransport' => [
        'default' => [
            'className' => 'Cake\Mailer\Transport\MailTransport',
            /*
             * The following keys are used in SMTP transports:
             */
            'host' => 'localhost',
            'port' => 25,
            'timeout' => 30,
            'username' => null,
            'password' => null,
            'client' => null,
            'tls' => null,
            'url' => env('EMAIL_TRANSPORT_DEFAULT_URL', null),
        ],
        'smtp' => [
            'host' => env('SMTP_HOST'),
            'port' => env('SMTP_PORT'),
            'timeout' => 60,
            'username' => env('SMTP_API_USER'),
            'password' => env('SMTP_API_KEY'),
            'className' => 'Smtp'
        ]
    ],

    /**
     * Email delivery profiles
     *
     * Delivery profiles allow you to predefine various properties about email
     * messages from your application and give the settings a name. This saves
     * duplication across your application and makes maintenance and development
     * easier. Each profile accepts a number of keys. See `Cake\Mailer\Email`
     * for more information.
     */
    'Email' => [
        'default' => [
            'transport' => 'default',
            'from' => 'you@localhost',
            //'charset' => 'utf-8',
            //'headerCharset' => 'utf-8',
        ],
        'smtp' => [
            'transport' => 'smtp',
            'from' => env('SMTP_FROM_EMAIL', 'you_need@to_configure.from_email')
        ]
    ],
];