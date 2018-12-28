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
        App.Websocket.onEvent('instancesUpdated', function (payload) {
            fillInstanceSelect(JSON.parse(payload.json));
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
        $.get('mustache/flashes.mst', function (template) {
            let flash = Mustache.render(template, {id: id, type: type, message: message});
            $('div.main').prepend(flash);
        });
        setTimeout(function () {
            $('#flash-' + id).hide()
        }, 5000);
    }
}

getPlaylists();
getInstances();