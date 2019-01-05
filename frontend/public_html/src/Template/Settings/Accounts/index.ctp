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

<div class="settings-container">
    <h1>Managing your account</h1>
    <hr>
    <br>
    <h3>Changing your password</h3>
    <?= $this->Form->create($user, ['url' => ['action' => 'changePassword']]) ?>
    <div class="row">
        <div class="small-12 medium-12 large-expand columns">
            <?= $this->Form->label('password', 'Old Password') ?>
            <?= $this->Form->password('password', ['minlength' => $minlength, 'class' => 'input radius', 'required'])
            ?>
        </div>
        <div class="small-12 medium-12 large-expand columns">
            <?= $this->Form->label('new_password', 'New Password') ?>
            <?= $this->Form->password('new_password', ['minlength' => $minlength,
            'placeholder' => 'Must have atleast ' . $minlength . ' characters', 'class' => 'input radius', 'required'])
            ?>
        </div>
        <div class="small-12 medium-12 large-expand columns">
            <?= $this->Form->label('new_password_repeat', 'Repeat New password') ?>
            <?= $this->Form->password('new_password_repeat', ['minlength' => $minlength,
            'placeholder' => 'Must have atleast ' . $minlength . ' characters', 'class' => 'input radius', 'required'])
            ?>
        </div>
    </div>
    <div class="row">
        <div class="columns">
            <?= $this->Form->button(__('Change Password'), ['class' => 'button expanded radius']); ?>
        </div>
    </div>
    <?= $this->Form->end() ?>
    <hr>
    <h3>Changing your email-address</h3>
    <?= $this->Form->create($user, ['url' => ['action' => 'changeEmail']]) ?>
    <div class="row">
        <div class="small-12 medium-12 large-expand columns">
            <?= $this->Form->control('password', ['minlength' => $minlength, 'class' => 'input radius', 'required']) ?>
        </div>
        <div class="small-12 medium-12 large-expand columns">
            <?= $this->Form->control('new_email', ['placeholder' => 'example@yamba.mail', 'class' => 'input radius',
            'required']) ?>
        </div>
    </div>
    <div class="row">
        <div class="columns">
            <?= $this->Form->button(__('Change Email'), ['class' => 'button expanded radius']); ?>
        </div>
    </div>
    <?= $this->Form->end() ?>
    <hr>
    <h3>Deleting your account</h3>
    <?= $this->Form->create($user, ['url' => ['action' => 'deleteAccount']]) ?>
    <div class="row">
        <div class="columns">
            <?= $this->Form->control('email', ['class' => 'input radius', 'required']) ?>
        </div>
    </div>
    <div class="row">
        <div class="columns">
            <?= $this->Form->button(__('Delete Account'), ['class' => 'button alert expanded radius']); ?>
        </div>
    </div>
    <?= $this->Form->end() ?>
</div>