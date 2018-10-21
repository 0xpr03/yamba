<?= $this->Form->create() ?>
<div class="row">
    <fieldset class="fieldset">
        <legend><?= __('Please enter your username and password') ?></legend>
        <div class="medium-6 cell">
            <?= $this->Form->control('email', ['placeholder' => 'example@mail.net']) ?>
        </div>
        <div class="medium-6 cell">
            <?= $this->Form->control('password', ['placeholder' => 'Not 1234']) ?>
        </div>
        <?= $this->Form->button(__('Login'), ['class' => 'button expanded']); ?>
    </fieldset>
</div>
<?= $this->Form->end() ?>