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
?>

<nav class="top-bar" data-topbar role="navigation">
    <div class="top-bar-left">
        <ul class="menu" data-dropdown-menu>
            <li>
                <a href="#" class="logo-font" onclick="fetchContent('/', 'music')">
                    <img src="/img/logo-silhouette.svg">
                </a>
            </li>
        </ul>
    </div>
    <div class="top-bar-right">
        <ul class="menu">
            <?php if($this->request->getSession()->read('Auth.User')) { ?>
            <li class="divider">
                <a href="#" class="header-font" onclick="fetchContent('/accounts/settings', 'account-settings')" title="Account Settings"><i class="fi-widget"></i></a>
            </li>
            <li class="divider"><?= $this->Html->link(
                'Logout',
                ['controller' => 'Users', 'action' => 'logout'],
                ['class' => 'header-font', 'title' => 'Logout']);
                ?>
            </li>
            <?php } ?>
        </ul>
    </div>
</nav>