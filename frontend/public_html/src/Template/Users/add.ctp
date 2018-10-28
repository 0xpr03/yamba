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

<?= $this->Form->create($user) ?>
<div class="grid-container">
    <fieldset class="fieldset">
        <legend><?= __('Register') ?></legend>
        <div class="grid-x grid-margin-x">
            <div class="large-6 cell">
                <?= $this->Form->control('email', ['label' => ['class' => 'required', 'text' => 'Email'], 'placeholder' => 'example@yamba.mail', 'class' => 'input radius']) ?>
            </div>
            <div class="large-6 cell">
                <?= $this->Form->control('password', ['label' => ['class' => 'required', 'text' => 'Password'], 'minlength' => $minlength, 'placeholder' => 'Must have atleast ' . $minlength . ' characters', 'class' => 'input radius']) ?>
            </div>
            <div class="large-6 cell">
                <?= $this->Html->link(
                'Already have an account? Sign in!',
                ['controller' => 'Users', 'action' => 'login'],
                ['class' => 'button expanded hollow success radius', 'type' => 'button']);
                ?>
            </div>
            <div class="large-6 cell">
                <?= $this->Form->button(__('Register'), ['class' => 'button expanded radius']); ?>
            </div>
        </div>
    </fieldset>
</div>