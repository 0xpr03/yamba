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

<div class="grid-container" style="margin-top: 2em;">
    <table class="hover">
        <thead>
        <tr>
            <th colspan="3" data-open="add-playlist-modal" style="font-weight: bold; padding: 0"><button class="button expanded" style="margin-bottom:0;width: 100%; height: 100%"><?= __('New Playlist') ?></button>
                <div class="reveal small" id="add-playlist-modal" data-reveal>
                    <?= $this->Form->create(null, ['id' => 'add-playlist-form']) ?>
                    <div class="grid-container">
                        <fieldset class="fieldset">
                            <legend><?= __('Create Playlist') ?></legend>
                            <div class="grid-x grid-margin-x">
                                <div class="large-6 cell">
                                    <?= $this->Form->control('name', ['label' => ['class' => 'required', 'text' => 'Name'],
                                    'id' => 'new-playlist-name', 'class' => 'input radius', 'placeholder' => 'Name of new playlist', 'required']) ?>
                                </div>
                                <div class="large-6 cell">
                                    <?= $this->Form->control('url', ['label' => ['text' => 'Url (optional)'], 'id' => 'new_playlist_url',
                                    'class' => 'input radius', 'placeholder' => 'URL to download titles (e.g. youtube playlist link)']) ?>
                                </div>
                                <div class="large-3 cell">
                                    <button id="close-add-playlist-modal" class="button hollow expanded secondary radius" data-close aria-label="Close modal" type="button">
                                        <span aria-hidden="true">Close</span>
                                    </button>
                                </div>
                                <div class="large-9 cell">
                                    <?= $this->Form->submit('Create Playlist', ['class' => 'button expanded radius']); ?>
                                </div>
                            </div>
                        </fieldset>
                    </div>
                    <?= $this->Form->end() ?>
                </div>
            </th>
        </tr>
        </thead>
        <tbody id="playlist-table">
        </tbody>
    </table>
</div>