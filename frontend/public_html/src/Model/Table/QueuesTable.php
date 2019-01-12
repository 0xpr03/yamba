<?php
/**
 * Created by PhpStorm.
 * User: tony
 * Date: 1/12/19
 * Time: 6:53 PM
 */

namespace App\Model\Table;

use Cake\ORM\Table;

class QueuesTable extends Table
{
    public function initialize(array $config)
    {
        parent::initialize($config);
        $this->belongsTo('Titles');
        $this->belongsTo('Instances');
    }
}