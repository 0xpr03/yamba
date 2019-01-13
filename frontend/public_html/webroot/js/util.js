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

function initYamba() {
    getInstances();
}

function hiliteTableRow(tbodySelector, playlist, attribute) {
    let tableRows = tbodySelector.find('tr');
    tableRows.each(function (index, item) {
            let classList = item.classList;
            let style = item.style;
            if (item.getAttribute(attribute) === playlist) {
                classList.add('black');
                style.color = '#fefefe';
            } else {
                classList.remove('black');
                style.color = '#0a0a0a';
            }
        }
    );
}

function selectedTrAttr(tbodySelector, attribute) {
    return tbodySelector.find('tr.black').attr(attribute);
}

function mapLengthFancy(titles) {
    titles.forEach((titles) => {
        titles.length = fancyTimeFormat(titles.length);
    });
    return titles;
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

function flash(type, message) {
    if (message !== undefined) {
        let id = guid();
        $.get('mustache/flashes.mst', function (template) {
            let flash = Mustache.render(template, {id: id, type: type, message: message});
            $('div.main').prepend(flash);
        });
        setTimeout(function () {
            $('#flash-' + id).hide()
        }, 5000);
    }
}