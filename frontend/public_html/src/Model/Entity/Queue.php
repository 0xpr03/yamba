<?php
/**
 * Created by PhpStorm.
 * User: tony
 * Date: 1/12/19
 * Time: 6:54 PM
 */

namespace App\Model\Entity;

use Cake\ORM\Entity;

class Queue extends Entity
{
    // Make all fields mass assignable except for primary key field "id".
    protected $_accessible = [
        '*' => true,
        'id' => false
    ];
}