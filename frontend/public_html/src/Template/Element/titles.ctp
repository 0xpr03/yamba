<?php
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
?>

<table id="titles-table" class="hover">
    <thead>
    <tr>
        <th>Name</th>
        <th>Artist</th>
        <th>Length</th>
        <th></th>
        <th></th>
    </tr>
    </thead>
    <tbody data-playlist-id="queue"></tbody>
    <script id="titles-table-body-template" type="x-tmpl-mustache">
<tbody class="playlist-titles" data-playlist-id="{{playlist}}">
{{#titles}}
    <tr class="pointer" onclick="/*TODO: play title*/" data-title-id="{{id}}">
        <td>{{name}}</td>
        <td>{{artist}}</td>
        <td>{{length}}</td>
        <td class="title-button">
           <a href="#" onclick="event.stopPropagation(); /*TODO configure title*/">
                <i class="fi-list"></i>
            </a>
        </td>
        <td class="title-button">
           <a href="#" onclick="$(this).closest('tr').hide(); event.stopPropagation(); deleteTitle('{{playlist}}', '{{id}}')">
               <span aria-hidden="true">&times;</span>
           </a>
        </td>
    </tr>
{{/titles}}
</tbody>
    </script>
</table>