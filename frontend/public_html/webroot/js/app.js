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

$(function () {
    $(document).foundation();

    let form = $('#add-playlist-form');
    form.submit(function (event) {
        event.preventDefault();
        addPlaylist(form);
    });
    App.Websocket.onEvent('playlistsUpdated', function (payload) {
        fillPlaylistTable(JSON.parse(payload.json));
    }.bind(this));
    getPlaylists();
});

function fetchContent(url, contentId) {
    $.ajax({
        method: 'get',
        url: url,
        success: function (response) {
            let mainContentDiv = $('#content');
            let contentDiv = $('#' + contentId);
            mainContentDiv.parent().children().hide();
            if (contentDiv.length) {
                contentDiv.show();
            } else {
                mainContentDiv.after('<div id="' + contentId + '">' + response + '</div>')
            }
        },
    });
}