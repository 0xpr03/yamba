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
    tableBody.html(Mustache.render(playlistsTemplate, playlists));
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
    flash('success', response);
}

function ajaxErrorFlash(response) {
    flash('alert', response);
}
