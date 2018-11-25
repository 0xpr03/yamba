App.Controllers.MusicIndexController = Frontend.AppController.extend({
    startup: function () {
        console.log('Startup');
        App.Websocket.onEvent('playlistsUpdated', function (payload) {
            fillPlaylistTable(JSON.parse(payload.json));
            if (payload.message != null && this.getVar('userID') === payload.userID) {
                let flash = $('#websocket-flash-div');
                if (payload.type != null) {
                    flash.addClass(payload.type);
                }
                flash.find('#websocket-flash-span').text(payload.message);
                flash.show();
            }
        }.bind(this));
    }
});
