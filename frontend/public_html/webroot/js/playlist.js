function fillPlaylistTable(playlists) {
    let tableBody = $('#playlist-table');
    let content = "";
    playlists.forEach((playlist) => {
        content +=
            '<tr>' +
            ' <td>' +
            '  <a href="#" onclick="deletePlaylist(\'' + playlist.id + '\')">' +
            '   <i style="color: red" class="fi-trash"></i>' +
            '  </a>' +
            ' </td>' +
            ' <td>' + playlist.name + '</td>' +
            ' <td>' +
            '  <span class="badge badge-right">' + playlist.titles_to_playlists.length + '</span>' +
            ' </td>' +
            '</tr>';
    });
    tableBody.html(content);
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
        error: function (response) {
            console.log(response);
            let errordiv = $('#add-playlist-error-div');
            errordiv.show();
            errordiv.find('#add-playlist-error-span').text(response.responseText);
        }
    });
}