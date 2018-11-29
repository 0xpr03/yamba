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
            '<tr>' +
            ' <td>' + title.name + '</td>' +
            ' <td>' + (title.artist == null ? "" : title.artist) + '</td>' +
            ' <td>' + fancyTimeFormat(title.length) + '</td>' +
            ' <td style="min-width: 32px; width: 32px; text-align: right">' +
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
            '<tr onclick="getTitles(\'' + playlist.id + '\')" style="cursor: pointer;">' +
            ' <td>' +
            '  <a href="#" onclick="event.stopPropagation(); deletePlaylist(\'' + playlist.id + '\')">' +
            '   <i style="color: red" class="fi-trash"></i>' +
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
    let successdiv = $('#add-playlist-success-div');
    let errordiv = $('#add-playlist-error-div');
    $.ajax({
        method: 'get',
        url: '/Music/addPlaylist',
        data: {'name': formData.name, 'url': formData.url},
        success: function (response) {
            if (response === 'OK') {
                $('#close-add-playlist-modal').click();
            } else {
                errordiv.hide();
                successdiv.find('#add-playlist-success-span').text(response);
                successdiv.show();
            }
            form.find('input[type=text]').val('');
        },
        error: function (response) {
            console.log(response);

            successdiv.hide();
            errordiv.find('#add-playlist-error-span').text(response.responseText);
            errordiv.show();
        }
    });
}

function deleteTitle(playlist, title) {
    $.ajax({
        method: 'get',
        url: '/Music/deleteTitle/' + playlist + '/' + title,
        success: function (response) {
        },
    });
}

function deletePlaylist(playlist) {
    $.ajax({
        method: 'get',
        url: '/Music/deletePlaylist',
        data: {'id': playlist},
        success: function (response) {
        },
    });
}
