<nav class="top-bar" data-topbar role="navigation">
    <div class="top-bar-left">
        <ul class="menu" data-dropdown-menu>
            <li class="menu-text menu-text-top"><?= $this->fetch('title') ?></li>
        </ul>
    </div>
    <div class="top-bar-right">
        <ul class="menu">
            <li class="divider"><?= $this->Html->link(
                '<i class="fi-widget"></i>',
                ['controller' => 'Accounts', 'action' => 'settings'],
                ['class' => 'button', 'title' => 'Account Settings', 'escapeTitle' => false]);
                ?></li>
            <li class="divider"><?= $this->Html->link(
                'Logout',
                ['controller' => 'Users', 'action' => 'logout'],
                ['class' => 'button', 'title' => 'Logout']);
                ?>
            </li>
        </ul>
    </div>
</nav>