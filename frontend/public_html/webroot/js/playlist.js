function getTitles(playlist) {
    $.ajax({
        method: 'get',
        url: '/Music/getTitles/' + playlist,
        success: function (response) {
            fillSongTable(playlist, response);
        },
    });
}

function fillSongTable(playlist, titles) {
    let tableBody = $('#titles-table-body');
    tableBody.attr('data-playlist-id', playlist);
    let content = "";
    titles.forEach((title) => {
        content +=
            '<tr class="pointer" onclick="/*TODO play title*/">' +
            ' <td>' + title.name + '</td>' +
            ' <td>' + (title.artist == null ? "" : title.artist) + '</td>' +
            ' <td>' + fancyTimeFormat(title.length) + '</td>' +
            ' <td class="title-button">' +
            '  <a href="#" onclick="event.stopPropagation(); deleteTitle(\'' + playlist + '\', \'' + title.id + '\')">' +
            '   <i class="fi-list"></i>' +
            '  </a>' +
            ' </td>' +
            ' <td class="title-button">' +
            '  <a href="#" onclick="event.stopPropagation(); deleteTitle(\'' + playlist + '\', \'' + title.id + '\')">' +
            '   <span aria-hidden="true">&times;</span>' +
            '  </a>' +
            ' </td>' +
            '</tr>';
    });
    tableBody.html(content);
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
    });
}

function fillPlaylistTable(playlists) {
    let tableBody = $('#playlist-table-body');
    let content = "";
    playlists.forEach((playlist) => {
        content +=
            '<tr class="pointer" onclick="getTitles(\'' + playlist.id + '\')">' +
            ' <td>' +
            '  <a href="#" onclick="event.stopPropagation(); deletePlaylist(\'' + playlist.id + '\')">' +
            '   <i class="fi-trash red-trash"></i>' +
            '  </a>' +
            ' </td>' +
            ' <td>' + playlist.name + '</td>' +
            ' <td>' +
            '  <span class="badge badge-right">' + (playlist.hasToken === "1" ? '<i class="fi-refresh large"></i>' : playlist.titles) + '</span>' +
            ' </td>' +
            '</tr>';
    });
    tableBody.html(content);
}

function addPlaylist(form) {
    let formData = form.serializeArray().reduce(function (obj, item) {
        obj[item.name] = item.value;
        return obj;
    }, {});
    $.ajax({
        method: 'get',
        url: '/Music/addPlaylist',
        data: {'name': formData.name, 'url': formData.url},
        success: function (response) {
            ajaxSuccessFlash(response);
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

function ajaxSuccessFlash(response) {
    flash('success', response.responseText);
}

function ajaxErrorFlash(response) {
    console.log(response);
    flash('alert', response.responseText);
}
