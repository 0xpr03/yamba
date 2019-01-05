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

function renderInstancesWithData() {
    getInstances(function (response) {
        fillInstanceSelect(response);
        renderInstanceData();
    }, function (response) {
        flash('alert', 'Unable to fetch instances');
    });
}

function renderInstances() {
    getInstances(function (response) {
        fillInstanceSelect(response);
    }, function (response) {
        flash('alert', 'Unable to fetch instances');
    });
}

function fillInstanceSelect(instances) {
    let instanceSelect = $('#instance-select');
    $.get('/mustache/instances_navbar.mst', function (template) {
        instanceSelect.html(Mustache.render(template, {instances: instances}));
    });
}

function getInstances(successCallback, errorCallback) {
    $.ajax({
        method: 'get',
        url: '/settings/Instances/getInstances',
        success: function (response) {
            successCallback(response);
        },
        error: function (response) {
            errorCallback(response);
        }
    });
}

function renderInstanceData() {
    getInstances(
        function (instances) {
            let instance = instances.filter(function (instance) {
                return instance.id === parseInt($('#instance-select option:selected').val());
            })[0];
            $('#instance-id').val(instance.id);
            $('#instance-name').val(instance.name);
            $('#instance-type').val(instance.type).change();
            switch (instance.type) {
                case 'teamspeak_instances':
                    let teamspeak = instance['teamspeak_instance'];
                    $('#teamspeak-host').val(teamspeak.host);
                    $('#teamspeak-identity').val(teamspeak.identity);
                    //$('#teamspeak-cid').val(teamspeak.cid).change();
                    if(teamspeak.hashed_password) {
                        $('#teamspeak-password').val(teamspeak.hashed_password);
                    }
                default:
                    break;
            }
        },
        function (response) {
            flash('alert', 'Unable to fetch instance data');
        }
    );

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