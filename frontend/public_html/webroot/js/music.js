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

function getPlaylists() {
    $.ajax({
        url: '/Music/getPlaylists',
        success: function (data) {
            $.get('/Music/getQueue/' + instanceSelect().val(), function (queue) {
                renderPlaylists(data, queue);
            });
        },
        error: function (data) {
            flash('alert', 'Unable to fetch playlists');
        },
    });
}

function getQueueTitles() {
    $.ajax({
        url: 'Music/getQueueTitles/' + instanceSelect().val(),
        success: function (data) {
            titles = data.map(function (entry) {
                return entry.title;
            });
            renderTitles(titles, 'queue');
            $('#playlist-id').val('queue');
        },
        error: function (data) {
            flash('alert', 'Unable to fetch queue titles');
        }
    });
}

function renderPlaylists(playlists, queue) {
    let tableBody = $('#playlist-table-body');
    let currentPlaylist = selectedTrAttr($('#playlist-table-body'), 'data-playlist-id');
    tableBody.html(Mustache.render(
        $('#playlist-table-body-template').html(),
        {
            playlists: playlists,
            queue: {
                length: queue
            }
        }
    ));
    if (Object.is(currentPlaylist, undefined) || $(`#playlist-table-body > tr[data-playlist-id="${currentPlaylist}"]`).length === 0) {
        hiliteTableRow(tableBody, 'queue', 'data-playlist-id');
        getQueueTitles();
    } else {
        hiliteTableRow(tableBody, currentPlaylist, 'data-playlist-id');
    }
}

function renderTitles(titles, playlist) {
    $(`#titles-table > tbody[data-playlist-id="${playlist}"]`).replaceWith(Mustache.render(
        $('#titles-table-body-template').html(),
        {
            titles: mapLengthFancy(titles),
            playlist: playlist
        }
    ));
}

function selectPlaylist(playlist) {
    hideTitles();
    let tbody = $(`#titles-table > tbody[data-playlist-id="${playlist}"]`);
    if (tbody.length > 0) {
        tbody.show();
    } else {
        $.ajax({
            url: '/Music/getTitles/' + playlist,
            success: function (data) {
                $('#titles-table').append(Mustache.render(
                    $('#titles-table-body-template').html(),
                    {
                        titles: mapLengthFancy(data),
                        playlist: playlist
                    }
                ));
            },
            error: function (data) {
                flash('alert', 'Unable to fetch titles');
            }
        });
    }
    hiliteTableRow($('#playlist-table-body'), playlist, 'data-playlist-id');
    $('#playlist-id').val(playlist);
}

function addTitle() {
    let currentPlaylist = selectedTrAttr($('#playlist-table-body'), 'data-playlist-id');
    let form = $('#add-title-form');
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
            form.find('input[type=reset]').click();
        },
        error: ajaxErrorFlash
    });
    $('#close-add-title-modal').click();
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
            form.find('input[type=reset]').click();
        },
        error: ajaxErrorFlash
    });
    $('#close-add-playlist-modal').click();
}

function deleteTitle(playlist, title) {
    $.ajax({
        url: `/Music/deleteTitle/${playlist}/${title}/${$('#instance-select').val()}`,
        error: function (response) {
            ajaxErrorFlash(response);
            $(`tbody[data-playlist-id="${playlist}"] > tr[data-title-id="${title}"]`).show();
        }
    });
}

function deletePlaylist(playlist) {
    $(`tbody[data-playlist-id="${playlist}"]`).hide();
    $.ajax({
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
    if (message.status === 404) {
        flash('alert', 'Unable to delete resource');
    }
}

function hideTitles() {
    $('#titles-table > tbody').hide();
}