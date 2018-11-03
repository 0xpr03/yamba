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

<?php $this->assign('title', $title); ?>
<?= $this->Form->create() ?>
<div class="credentials-container">
    <fieldset class="fieldset">
        <legend><?= __('Please enter your username and password') ?></legend>
        <div class="row">
            <div class="columns">
                <?= $this->Form->control('email', ['class' => 'input radius']) ?>
            </div>
            <div class="columns">
                <?= $this->Form->control('password', ['class' => 'input radius']) ?>
            </div>
        </div>
        <div class="row">
            <div class="columns">
                <?= $this->Html->link(
                'Don\'t have an account yet? Register here!',
                ['controller' => 'Users', 'action' => 'add'],
                ['class' => 'button expanded hollow alert radius', 'type' => 'button']);
                ?>
            </div>
            <div class="columns">
                <?= $this->Form->button(__('Login'), ['class' => 'button expanded radius']); ?>
            </div>
        </div>
    </fieldset>
</div>
<?= $this->Form->end() ?>