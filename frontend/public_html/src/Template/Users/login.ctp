<?= $this->Form->create() ?>
<div class="grid-container">
    <fieldset class="fieldset">
        <legend><?= __('Please enter your username and password') ?></legend>
        <div class="grid-x grid-margin-x">
            <div class="medium-12 large-6 cell">
                <?= $this->Form->control('email', ['class' => 'input radius']) ?>
            </div>
            <div class="medium-12 large-6 cell">
                <?= $this->Form->control('password', ['class' => 'input radius']) ?>
            </div>
            <div class="medium-12 large-6 cell">
                <?= $this->Html->link(
                'Don\'t have an account yet? Register here!',
                ['controller' => 'Users', 'action' => 'add'],
                ['class' => 'button expanded hollow alert radius', 'type' => 'button']);
                ?>
            </div>
            <div class="medium-12 large-6 cell">
                <?= $this->Form->button(__('Login'), ['class' => 'button expanded radius']); ?>
            </div>
        </div>
    </fieldset>
</div>
<?= $this->Form->end() ?>