<!DOCTYPE html>
<html>
<head>
    <?= $this->Html->charset() ?>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>
        <?= $this->fetch('title') ?> - Yamba
    </title>
    <?= $this->Html->meta('icon', 'img/favicon.ico', ['type'=>'icon']) ?>
    <?= $this->fetch('meta') ?>

    <?= $this->Html->css(['app', 'foundation.min', 'foundation-icons/foundation-icons']); ?>
    <?= $this->fetch('css') ?>

    <?= $this->Html->script(['jquery-3.3.1.min', 'vendor/foundation.min', 'vendor/what-input']); ?>
    <?= $this->fetch('script') ?>
</head>
<body>
<div class="grid-container">
    <?= $this->fetch('content') ?>
</div>
<footer>
    <div class="row">
        <span class="footer-copyright">© 2018–<?= date('Y')?> Yamba Authors</span>
        <a class="footer-icon" href="https://github.com/0xpr03/yamba" target="_blank"><i class="fi-social-github"></i></a>
    </div>
</footer>
<?= $this->Html->script('app'); ?>
</body>
</html>