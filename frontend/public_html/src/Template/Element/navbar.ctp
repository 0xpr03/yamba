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
                <a href="/" class="logo-font">
                    <img src="/img/logo-silhouette.svg">
                </a>
            </li>
        </ul>
    </div>
    <div class="top-bar-right">
        <ul class="dropdown menu" data-dropdown-menu>
            <?php if($this->request->getSession()->read('Auth.User')) { ?>
            <li>
                <a href="#" class="header-font">Settings</a>
                <ul class="menu" style="border-top: 0px">
                    <li><a href="/settings/accounts" class="header-font">Account Settings</a></li>
                    <li><a href="/settings/instances" class="header-font">Instance Settings</a></li>
                </ul>
            </li>
            <?php if($this->request->getParam('controller') === 'Music' || $this->request->getParam('controller') === 'Instances') { ?>
            <li>
                <select id="instance-select">
                </select>
            </li>
            <?php } ?>
            <li><?= $this->Html->link(
                'Logout',
                ['controller' => 'Users', 'action' => 'logout'],
                ['class' => 'header-font', 'title' => 'Logout']);
                ?>
            </li>
            <?php } ?>
        </ul>
    </div>
</nav>