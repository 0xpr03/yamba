<?php
/**
 * Created by PhpStorm.
 * User: tony
 * Date: 20.10.18
 * Time: 15:39
 */
namespace App\Model\Table;
use Cake\ORM\Table;
use Cake\Validation\Validator;
class UsersTable extends Table
{
    public function validationDefault(Validator $validator)
    {
        return $validator
            ->notEmpty('email', 'An email is required')
            ->notEmpty('password', 'A password is required');
    }
}