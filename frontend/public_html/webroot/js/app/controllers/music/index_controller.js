let flashTemplate;
let titlesTemplate;
let playlistsTemplate;

$.get('mustache/flash.mst', function(template) {
    flashTemplate = template;
    Mustache.parse(flashTemplate);
});

$.get('mustache/titles.mst', function(template) {
    titlesTemplate = template;
    Mustache.parse(titlesTemplate);
});

$.get('mustache/playlists.mst', function(template) {
    playlistsTemplate = template;
    Mustache.parse(playlistsTemplate);
});

App.Controllers.MusicIndexController = Frontend.AppController.extend({
    startup: function () {
        App.Websocket.onEvent('playlistsUpdated', function (payload) {
            fillPlaylistTable(JSON.parse(payload.json));
        }.bind(this));
        App.Websocket.onEvent('titlesUpdated', function (payload) {
            if ($('#titles-table-body').attr('data-playlist-id') === payload.playlist) {
                fillSongTable(payload.playlist, JSON.parse(payload.json));
            }
        }.bind(this));
        App.Websocket.onEvent('flash', function (payload) {
            if (this.getVar('userID') === payload.userID) {
                flash(payload.type, payload.message);
            }
        }.bind(this));
    }
});

function flash(type, message) {
    if (message !== undefined) {
        let id = guid();
        let flash = Mustache.render(flashTemplate, {id: id, type: type, message: message});
        $('div.main').prepend(flash);
        setTimeout(function () {
            $('#flash-' + id).hide()
        }, 5000);
    }
}
