<nav class="top-bar" data-topbar role="navigation">
    <div class="top-bar-left">
        <ul class="menu" data-dropdown-menu>
            <li class="menu-text menu-text-top"><?= $this->fetch('title') ?></li>
        </ul>
    </div>
    <div class="top-bar-right">
        <ul class="menu">
            <li><?= $this->Html->link(
                'Logout',
                ['controller' => 'Users', 'action' => 'logout'],
                ['class' => 'button']);
                ?>
            </li>
        </ul>
    </div>
</nav>