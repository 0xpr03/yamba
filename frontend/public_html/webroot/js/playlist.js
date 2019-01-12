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

function renderTitles(playlist) {
    $('.playlist-titles').hide();

    let titles = getPlaylistTitles(playlist);
    if (titles.length > 0) {
        titles.show();
    } else {
        $.ajax({
            method: 'get',
            url: (playlist === 'queue' ? 'Music/getQueueTitles/' + $('#instance-select').val() : '/Music/getTitles/' + playlist),
            success: function (response) {
                addTitleBody(playlist, response, true);
            },
            error: function (response) {
                flash('alert', 'Unable to fetch titles');
            }
        });
    }
    hiliteTableRow(playlist);
}

function getPlaylistTitles(playlist) {
    return $('.playlist-titles[data-playlist-id="' + playlist + '"]');
}

function addTitleBody(playlist, titles, show) {
    titles.forEach((title) => {
        title.length = fancyTimeFormat(title.length);
    });
    let titlesTable = $('#titles-table');
    $.get('mustache/titles.mst', function (template) {
        let playlistTitles = Mustache.render(template, {playlist: playlist, titles: titles});
        if (show) {
            titlesTable.append(playlistTitles);
        } else {
            getPlaylistTitles(playlist).replaceWith(playlistTitles);
        }
    });
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
            fillPlaylistTable(response, true);
        },
        error: function (response) {
            flash('alert', 'Unable to fetch playlists');
        }
    });
}

function fillPlaylistTable(playlists, renderQueue) {
    let tableBody = $('#playlist-table-body');
    let currentPlaylist = $('#playlist-table > tbody > tr.black').attr('data-playlist-id');

    $.get('/Music/getQueue/' + $('#instance-select').val(), function (queue) {
        $.get('mustache/playlists.mst', function (template) {
            tableBody.html(Mustache.render(template, {
                playlists: playlists,
                queue: {
                    length: queue
                }
            }));
            if (renderQueue) {
                renderTitles('queue');
            } else {
                hiliteTableRow(currentPlaylist);
            }
        });
    });
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
        method: 'get',
        url: '/Music/deleteTitle/' + playlist + '/' + title,
        error: function (response) {
            ajaxErrorFlash(response);
            $('tbody[data-playlist-id="' + playlist + '"] > tr[data-title-id="' + title + '"]').show();
        }
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
    if (message.status === 404) {
        flash('alert', 'Unable to delete resource');
    }
}

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
