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

namespace App\Model\Table;
use Cake\ORM\Table;
use Cake\Validation\Validator;
class AddTitlesJobsTable extends Table
{
    public function initialize(array $config)
    {
        parent::initialize($config);
        $this->hasOne('Playlists');
        $this->hasOne('Users');
    }

    public function validationDefault(Validator $validator)
    {
        return $validator
            ->notEmpty('backend_token', 'Must specify backend_token')
            ->notEmpty('playlist_id', 'Must specify playlist_id')
            ->notEmpty('user_id', 'Must specify user_id');
    }
}