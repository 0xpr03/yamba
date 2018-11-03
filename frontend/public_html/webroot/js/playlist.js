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
        },
    });
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
            $('#close-add-playlist-modal').click();
            form.find('input[type=text]').val('');
        },
    });
}