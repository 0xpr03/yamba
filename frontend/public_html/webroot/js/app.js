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
    App.Websocket.onEvent('playlistsUpdated', function (payload) {
        fillPlaylistTable(JSON.parse(payload.json));
    }.bind(this));
    fetchContent(window.location.pathname);
});

function fetchContent(url) {
    let contentId;
    if (url === '/') {
        contentId = 'music';
    } else {
        contentId = url.toLowerCase().replace(new RegExp('/', 'g'), '-').replace('-','');
    }
    let mainContentDiv = $('#content');
    let contentDiv = $('#' + contentId);
    mainContentDiv.children().hide();
    if (contentDiv.length) {
        contentDiv.show();
        window.history.pushState({},'',url);
    } else {
        $.ajax({
            method: 'get',
            url: url,
            success: function (response) {
                mainContentDiv.append('<div id="' + contentId + '">' + response + '</div>');
                window.history.pushState({},'',url);
            },
            error: function (response) {
                if (response.status === 403) {
                    fetchContent('/users/login');
                }
            }
        });
    }
}