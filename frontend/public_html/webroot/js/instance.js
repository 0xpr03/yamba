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
    $.ajax({
        url: '/settings/Instances/getInstances',
        success: function (data) {
            renderInstances(data);
        },
        error: function (data) {
            flash('alert', 'Unable to get instances');
        },
    }).always(function () {
        getPlaylists();
    });
}

function renderInstances(instances) {
    $('#instance-select').html(Mustache.render(
        $('#instance-select-template').html(),
        {instances: instances}
    ));
}

function instanceSelect() {
    return $('#instance-select');
}