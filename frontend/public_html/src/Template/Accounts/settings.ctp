<div class="grid-container">
    <h1>Managing your account</h1>
    <hr>
    <br>
    <h3>Changing your password</h3>
    <?= $this->Form->create($user, ['url' => ['action' => 'changePassword']]) ?>
    <div class="grid-x grid-padding-x">
        <div class="medium-12 large-4 cell">
            <?= $this->Form->label('password', 'Old Password') ?>
            <?= $this->Form->password('password', ['minlength' => $minlength, 'class' => 'input radius', 'required' => true]) ?>
        </div>
        <div class="medium-12 large-4 cell">
            <?= $this->Form->label('new_password', 'New Password') ?>
            <?= $this->Form->password('new_password', ['minlength' => $minlength, 'placeholder' => 'Must have atleast ' . $minlength . ' characters', 'class' => 'input radius', 'required' => true]) ?>
        </div>
        <div class="medium-12 large-4 cell">
            <?= $this->Form->label('new_password_repeat', 'Repeat New password') ?>
            <?= $this->Form->password('new_password_repeat', ['minlength' => $minlength, 'placeholder' => 'Must have atleast ' . $minlength . ' characters', 'class' => 'input radius', 'required' => true]) ?>
        </div>
        <div class="cell">
            <?= $this->Form->button(__('Change Password'), ['class' => 'button expanded radius']); ?>
        </div>
    </div>
    <?= $this->Form->end() ?>
    <hr>
    <h3>Changing your email-address</h3>
    <hr>
    <h3>Deleting your account</h3>
</div>