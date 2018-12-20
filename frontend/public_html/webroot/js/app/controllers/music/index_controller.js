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

let flashTemplate;
let titlesTemplate;
let playlistsTemplate;
let instancesTemplate;

$.get('mustache/flash.mst', function(template) {
    flashTemplate = template;
    Mustache.parse(flashTemplate);
});

$.get('mustache/titles.mst', function(template) {
    titlesTemplate = template;
    Mustache.parse(titlesTemplate);
});

$.get('mustache/playlists.mst', function(template) {
    playlistsTemplate = template;
    Mustache.parse(playlistsTemplate);
});

$.get('mustache/instances.mst', function(template) {
    instancesTemplate = template;
    Mustache.parse(instancesTemplate);
});

App.Controllers.MusicIndexController = Frontend.AppController.extend({
    startup: function () {
        App.Websocket.onEvent('playlistsUpdated', function (payload) {
            fillPlaylistTable(JSON.parse(payload.json));
        }.bind(this));
        App.Websocket.onEvent('titlesUpdated', function (payload) {
            if ($('#titles-table-body').attr('data-playlist-id') === payload.playlist) {
                fillSongTable(payload.playlist, JSON.parse(payload.json));
            }
        }.bind(this));
        App.Websocket.onEvent('flash', function (payload) {
            if (this.getVar('userID') === payload.userID) {
                flash(payload.type, payload.message);
            }
        }.bind(this));
    }
});

function flash(type, message) {
    if (message !== undefined) {
        let id = guid();
        let flash = Mustache.render(flashTemplate, {id: id, type: type, message: message});
        $('div.main').prepend(flash);
        setTimeout(function () {
            $('#flash-' + id).hide()
        }, 5000);
    }
}

getPlaylists();
let form = $('#add-playlist-form');
form.submit(function (event) {
    event.preventDefault();
    addPlaylist(form);
});