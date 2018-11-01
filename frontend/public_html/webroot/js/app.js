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

$(document).foundation();

$(function () {
    let form = $('#add-playlist-form');
    form.submit(function (event) {
        event.preventDefault();
        let formData = form.serializeArray().reduce(function (obj, item) {
            obj[item.name] = item.value;
            return obj;
        }, {});
        $.ajax({
            method: 'get',
            url: '/Music/addPlaylist',
            data: {'name': formData.name, 'url': formData.url},
            success: function (response) {
                fillPlaylistTable(response);
                $('#close-add-playlist-modal').click();
                form.find('input[type=text]').val('');
            },
        });
    });

    $.ajax({
        method: 'get',
        url: '/Music/getPlaylists',
        success: function (response) {
            fillPlaylistTable(response);
        },
    });

});

function fillPlaylistTable(playlists) {
    let tableBody = $('#playlist-table');
    tableBody.empty();
    playlists.forEach((playlist) => {
        tableBody.append('<tr><td><a href="#" onclick="deletePlaylist(\'' + playlist.id + '\')"><i style="color: red" class="fi-trash"></i></a></td><td>' + playlist.name + '<td><span class="badge badge-right">' + playlist.songs_to_playlists.length + '</span></td>' + '</td></tr>');
    });
}

function deletePlaylist(playlist) {
    $.ajax({
        method: 'get',
        url: '/Music/deletePlaylist',
        data: {'id': playlist},
        success: function (response) {
            fillPlaylistTable(response);
        },
    });
}