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
<div class="instances-container">
    <h1>Managing your Instance</h1>
    <hr>
    <br>
    <?= $this->Form->create(null, ['url' => ['action' => 'addInstance']]) ?>
    <div class="row">
        <div class="columns">
            <?= $this->Form->control('name', ['label' => ['class' => 'required', 'text' => 'Instance name'],
            'placeholder' => 'Yamba Music Bot', 'class' => 'input radius', 'required']) ?>
        </div>
        <div class="columns shrink">
            <?= $this->Form->control('type', ['label' => ['class' => 'required', 'text' => 'Instance type'],
            'class' => 'input radius', 'type' => 'select', 'onchange' => 'changeType()', 'id' => 'instance-type',
            'options' => ['teamspeak_instances' => 'Teamspeak', 'tbd' => 'TBD (e.g. discord)']]) ?>
        </div>
    </div>
    <div class="row">
        <div class="columns shrink">
            <?= $this->Form->control('autostart', ['type' => 'checkbox', 'label' => ['class' => 'required'],
            'placeholder' => 'Instance name', 'class' => 'input radius']) ?>
        </div>
    </div>
    <hr>
    <br>
    <div id="teamspeak-instances">
        <div class="row">
            <div class="columns">
                <?= $this->Form->control('host', ['label' => ['class' => 'required', 'text' => 'Teamspeak host'],
                'placeholder' => 'example.domain.net', 'class' => 'input radius', 'required']) ?>
            </div>
            <div class="columns">
                <?= $this->Form->control('identity', ['label' => ['class' => 'required', 'text' => 'Bot Identity'],
                'placeholder' => 'xxxxxxxxxxxxx',
                'class' => 'input radius', 'required']) ?>
            </div>
        </div>
        <div class="row">
            <div class="columns">
                <?= $this->Form->control('cid', ['label' => ['text' => 'Default Channel ID'],
                'class' => 'input radius', 'type' => 'select', 'onchange' => 'changeType()',
                'options' => ['1234' => 'example', '5678' => 'channel']]) ?>
            </div>
            <div class="columns">
                <?= $this->Form->control('port', ['label' => ['text' => 'Teamspeak port'],
                'placeholder' => 'example.domain.net', 'class' => 'input radius', 'required']) ?>
            </div>
            <div class="columns">
                <?= $this->Form->control('password', ['label' => ['text' => 'Teamspeak host'],
                'placeholder' => 'example.domain.net', 'class' => 'input radius', 'required']) ?>
            </div>
        </div>
    </div>
    <?= $this->Form->submit('Update instances', ['class' => 'button expanded radius']) ?>
    <?= $this->Form->end() ?>
</div>
<script>changeType()</script>