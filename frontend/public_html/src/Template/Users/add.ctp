<?= $this->Form->create($user) ?>
<div class="row">
    <fieldset class="fieldset">
        <legend><?= __('Add user') ?></legend>
        <div class="medium-6 cell">
            <?= $this->Form->control('email', ['label' => ['class' => 'required', 'text' => 'Email'], 'placeholder' => 'example@mail.net']) ?>
        </div>
        <div class="medium-6 cell">
            <?= $this->Form->control('password', ['label' => ['class' => 'required', 'text' => 'Password'], 'placeholder' => 'Please don\'t do 1234']) ?>
        </div>
        <?= $this->Form->button(__('Register'), ['class' => 'button expanded']); ?>
    </fieldset>
</div>
<?= $this->Form->end() ?>
<div class="row">
    <?= $this->Html->link(
    'Already have an account? Login here!',
    ['controller' => 'Users', 'action' => 'login'],
    ['class' => 'button expanded', 'type' => 'button']);
    ?>
</div>