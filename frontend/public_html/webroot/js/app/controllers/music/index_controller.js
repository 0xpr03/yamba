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
        let flash = $('#websocket-flash-div');
        flash.addClass(type);
        flash.find('#websocket-flash-span').text(message);
        flash.show();
        setTimeout(function () {
            flash.hide()
        }, 5000);
    }
}
