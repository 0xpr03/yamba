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