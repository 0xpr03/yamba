function fillPlaylistTable(playlists) {
    let tableBody = $('#playlist-table-body');
    let content = "";
    playlists.forEach((playlist) => {
        content +=
            '<tr onclick="renderPlaylist(\'' + playlist.id + '\')" style="cursor: pointer;">' +
            ' <td>' +
            '  <a href="#" onclick="event.stopPropagation(); deletePlaylist(\'' + playlist.id + '\')">' +
            '   <i style="color: red" class="fi-trash"></i>' +
            '  </a>' +
            ' </td>' +
            ' <td>' + playlist.name + '</td>' +
            ' <td>' +
            '  <span class="badge badge-right">' + playlist.titles + '</span>' +
            ' </td>' +
            '</tr>';
    });
    tableBody.html(content);
}

function renderPlaylist(id) {
    /*$.ajax({
        method: 'get',
        
    });*/
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