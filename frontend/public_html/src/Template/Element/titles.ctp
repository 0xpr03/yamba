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

<table id="titles-table" class="hover">
    <thead>
    <tr>
        <th>Name</th>
        <th>Artist</th>
        <th>Length</th>
        <th colspan="2" style="padding: 0">
            <button class="button expanded" data-open="add-title-modal"
                    style="margin-bottom:0"><?= __('Add') ?></button>
            <div class="reveal small" id="add-title-modal" data-reveal>
                <?= $this->Form->create(null, ['id' => 'add-title-form', 'url' => 'Music/addTitle',
                'onsubmit' => 'event.preventDefault(); addTitle();']) ?>
                <fieldset class="fieldset">
                    <legend><?= __('Add title(s)') ?></legend>
                    <div class="row">
                        <div class="columns">
                            <?= $this->Form->control('url', ['label' => ['class' => 'required', 'text' => 'Url'],
                            'id' => 'title-url', 'class' => 'input radius', 'required',
                            'placeholder' => 'URL to download title (e.g. youtube video link)']) ?>
                        </div>
                    </div>
                    <?= $this->Form->hidden('playlist-id', ['id' => 'playlist-id', 'default' => '-1']); ?>
                    <?= $this->Form->hidden('instance-id', ['id' => 'instance-id', 'default' => '-1']); ?>
                    <?= $this->Form->unlockField('playlist-id'); ?>
                    <?= $this->Form->unlockField('instance-id'); ?>
                    <div class="row">
                        <div class="columns shrink">
                            <button id="close-add-title-modal" class="button hollow alert radius"
                                    data-close aria-label="Close modal" type="button">
                                <span aria-hidden="true">Close</span>
                            </button>
                        </div>
                        <div class="columns shrink">
                            <?= $this->Form->submit('Reset', ['type' => 'reset',
                            'class' => 'button warning hollow radius']); ?>
                        </div>
                        <div class="columns">
                            <?= $this->Form->submit('Add Title', ['class' => 'button expanded radius']); ?>
                        </div>
                    </div>
                </fieldset>
                <?= $this->Form->end() ?>
            </div>
        </th>
    </tr>
    </thead>
    <tbody data-playlist-id="queue"></tbody>
    <script id="titles-table-body-template" type="x-tmpl-mustache">
<tbody class="playlist-titles" data-playlist-id="{{playlist}}">
{{#titles}}
    <tr class="pointer" onclick="/*TODO: play title*/" data-title-id="{{id}}">
        <td>{{name}}</td>
        <td>{{artist}}</td>
        <td>{{length}}</td>
        <td class="title-button">
           <a href="#" onclick="event.stopPropagation(); /*TODO configure title*/">
                <i class="fi-list"></i>
            </a>
        </td>
        <td class="title-button">
           <a href="#" onclick="$(this).closest('tr').hide(); event.stopPropagation(); deleteTitle('{{playlist}}', '{{id}}')">
               <span aria-hidden="true">&times;</span>
           </a>
        </td>
    </tr>
{{/titles}}
</tbody>
    </script>
</table>