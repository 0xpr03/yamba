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
    <?= $this->Html->meta('icon') ?>
    <?= $this->fetch('meta') ?>

    <?= $this->Html->css(['app', 'foundation.min', 'foundation-icons/foundation-icons']); ?>
    <?= $this->fetch('css') ?>

    <?= $this->Html->script(['jquery-3.3.1.min', 'vendor/foundation.min', 'vendor/what-input']); ?>
    <?= $this->fetch('script') ?>
</head>
<body>
<nav class="top-bar" data-topbar role="navigation">
    <div class="top-bar-left">
        <ul class="menu" data-dropdown-menu>
            <li class="menu-text menu-text-top"><?= $this->fetch('title') ?></li>
        </ul>
    </div>
    <div class="top-bar-right">
        <ul class="menu">
            <li><a type="button" class="button" title="Start/Stop Yamba"><i class="fi-power"></i></a></li>
            <li class="divider"></li>
            <li><?= $this->Html->link(
                'Logout',
                ['controller' => 'Users', 'action' => 'logout'],
                ['class' => 'button']);
                ?>
            </li>
        </ul>
    </div>
</nav>
<?= $this->Flash->render() ?>
<div class="grid-container">
    <?= $this->fetch('content') ?>
</div>
<footer>
    <div class="row">
        <span class="footer-copyright">© 2018–<?= date('Y')?> Yamba Authors</span>
        <a class="footer-icon" href="https://github.com/0xpr03/yamba"><i class="fi-social-github"></i></a>
    </div>
</footer>
<?= $this->Html->script('app'); ?>
</body>
</html>
