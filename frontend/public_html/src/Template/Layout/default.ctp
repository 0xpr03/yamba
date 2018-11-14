<?php
/**
 * CakePHP(tm) : Rapid Development Framework (https://cakephp.org)
 * Copyright (c) Cake Software Foundation, Inc. (https://cakefoundation.org)
 *
 * Licensed under The MIT License
 * For full copyright and license information, please see the LICENSE.txt
 * Redistributions of files must retain the above copyright notice.
 *
 * @copyright     Copyright (c) Cake Software Foundation, Inc. (https://cakefoundation.org)
 * @link          https://cakephp.org CakePHP(tm) Project
 * @since         0.10.0
 * @license       https://opensource.org/licenses/mit-license.php MIT License
 */

$cakeDescription = 'CakePHP: the rapid development php framework';
?>
<!DOCTYPE html>
<html>
<head>
    <?= $this->Html->charset() ?>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>
        <?= $this->fetch('title') ?> - Yamba
    </title>
    <?= $this->Html->meta('icon', 'favicon.ico', ['type'=>'icon']) ?>
    <?= $this->fetch('meta') ?>

    <?= $this->Html->css(['app', 'foundation.min', 'foundation-icons/foundation-icons']); ?>
    <?= $this->fetch('css') ?>

    <?= $this->Html->script(['vendor/jquery', 'vendor/foundation.min', 'vendor/what-input', 'playlist']); ?>

    <?php if(isset($this->FrontendBridge)) {
    $this->FrontendBridge->init($frontendData);
    echo $this->FrontendBridge->run();
    } ?>
    <?= $this->Html->script(['webroot/lib/websocket.js']); ?>
    <?= $this->fetch('script') ?>
</head>
<body>
<?= $this->element('navbar'); ?>
<div<?= $this->FrontendBridge->getControllerAttributes() ?>>
    <div class="main">
        <?= $this->Flash->render() ?>
        <div id="content">
        </div>
    </div>
</div>
<footer>
    <span class="footer-copyright">© 2018–<?= date('Y')?> Yamba Authors</span>
    <a class="footer-icon" href="https://github.com/0xpr03/yamba" target="_blank"><i class="fi-social-github"></i></a>
</footer>
<?= $this->Html->script('app'); ?>
</body>
</html>
