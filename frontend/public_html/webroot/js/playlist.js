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

function hiliteTableRow(playlist) {
    let tableRows = $('#playlist-table > tbody > tr');
    tableRows.each(function (index, item) {
            let classList = item.classList;
            let style = item.style;
            if (item.getAttribute('data-playlist-id') === playlist) {
                classList.add('black');
                style.color = '#fefefe';
            } else {
                classList.remove('black');
                style.color = '#0a0a0a';
            }
        }
    );
}

function getTitles(playlist) {
    $.ajax({
        method: 'get',
        url: '/Music/getTitles/' + playlist,
        success: function (response) {
            fillSongTable(playlist, response);
        },
        error: function (response) {
            flash('alert', 'Unable to fetch titles');
        }
    });
    hiliteTableRow(playlist);
}

function fillSongTable(playlist, titles) {
    let tableBody = $('#titles-table-body');
    tableBody.attr('data-playlist-id', playlist);
    titles.forEach((title) => {
        title.length = fancyTimeFormat(title.length);
    });
    tableBody.html(Mustache.render(titlesTemplate, {playlist: playlist, titles: titles}));
}

function fancyTimeFormat(time) {
    // Hours, minutes and seconds
    var hrs = ~~(time / 3600);
    var mins = ~~((time % 3600) / 60);
    var secs = ~~time % 60;

    // Output like "1:01" or "4:03:59" or "123:03:59"
    var ret = "";

    if (hrs > 0) {
        ret += "" + hrs + ":" + (mins < 10 ? "0" : "");
    }

    ret += "" + mins + ":" + (secs < 10 ? "0" : "");
    ret += "" + secs;
    return ret;
}

function getPlaylists() {
    $.ajax({
        method: 'get',
        url: '/Music/getPlaylists',
        success: function (response) {
            fillPlaylistTable(response);
        },
        error: function (response) {
            flash('alert', 'Unable to fetch playlists');
        }
    });
}

function fillPlaylistTable(playlists) {
    let tableBody = $('#playlist-table-body');
    tableBody.html(Mustache.render(playlistsTemplate, {playlists: playlists}));
    hiliteTableRow($('#titles-table-body').attr('data-playlist-id'));
}

function addPlaylist() {
    let form = $('#add-playlist-form');
    let formData = form.serializeArray().reduce(function (obj, item) {
        obj[item.name] = item.value;
        return obj;
    }, {});
    $.ajax({
        method: 'post',
        url: form.attr('action'),
        beforeSend: function (xhr) {
            xhr.setRequestHeader('X-CSRF-Token', $('[name="_csrfToken"]').val());
        },
        data: formData,
        success: function (message, status, jqXHR) {
            ajaxSuccessFlash(message, jqXHR.status);
            form.find('input[type=text]').val('');
        },
        error: ajaxErrorFlash
    });
    $('#close-add-playlist-modal').click();
}

function deleteTitle(playlist, title) {
    $.ajax({
        method: 'get',
        url: '/Music/deleteTitle/' + playlist + '/' + title,
        error: ajaxErrorFlash
    });
}

function deletePlaylist(playlist) {
    $.ajax({
        method: 'get',
        url: '/Music/deletePlaylist',
        data: {'id': playlist},
        error: ajaxErrorFlash
    });
}

function ajaxSuccessFlash(message, statusCode) {
    let status = '';
    if (statusCode === 201) {
        status = 'success';
    } else if (statusCode === 202) {
        status = 'warning';
    } else {
        status = 'primary';
    }
    flash(status, message);
}

function ajaxErrorFlash(message) {
    flash('alert', message);
}
