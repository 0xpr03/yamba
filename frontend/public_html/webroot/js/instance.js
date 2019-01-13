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

function getInstances() {
    return instanceAjax().done(function (data) {
        renderInstances(data);
    });
}

function instanceAjax() {
    return $.ajax({
        url: '/settings/Instances/getInstances',
        error: function (data) {
            flash('alert', 'Unable to get instances');
        },
    });
}

function selectInstance() {
    return instanceAjax().done(function (data) {
        renderInstanceData(data);
        getPlaylists();
    });
}

function renderInstances(instances) {
    $('#instance-select').html(Mustache.render(
        $('#instance-select-template').html(),
        {instances: instances}
    ));
}

function renderInstanceData(instances) {
    let instance = instances.filter(function (instance) {
        return instance.id === parseInt($('#instance-select option:selected').val());
    })[0];
    $('#instance-id').val(instance.id);
    $('#instance-name').val(instance.name);
    $('#instance-type').val(instance.type).change();
    $('#instance-autostart').prop('checked', instance.autostart);
    switch (instance.type) {
        case 'teamspeak_instances':
            let teamspeak = instance['teamspeak_instance'];
            $('#teamspeak-host').val(teamspeak.host);
            $('#teamspeak-identity').val(teamspeak.identity);
            //$('#teamspeak-cid').val(teamspeak.cid).change();
            if (teamspeak.hashed_password) {
                $('#teamspeak-password').val(teamspeak.hashed_password);
            }
        default:
            break;
    }
}

function changeType() {
    let teamspeakContainer = $('#teamspeak-instances');
    let containers = [teamspeakContainer];
    containers.forEach(function (container) {
        container.hide();
    });
    switch ($('#instance-type').val()) {
        case 'teamspeak_instances':
            teamspeakContainer.show();
            break;
        default:
            flash('warning', 'this instance type is not yet supported');
            break;
    }
}

function instanceSelect() {
    return $('#instance-select');
}