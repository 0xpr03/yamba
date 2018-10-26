<?= $this->Form->create($user) ?>
<div class="grid-container">
    <fieldset class="fieldset">
        <legend><?= __('Register') ?></legend>
        <div class="grid-x grid-margin-x">
            <div class="medium-12 large-6 cell">
                <?= $this->Form->control('email', ['label' => ['class' => 'required', 'text' => 'Email'], 'placeholder' => 'example@yamba.mail', 'class' => 'input radius']) ?>
            </div>
            <div class="medium-12 large-6 cell">
                <?= $this->Form->control('password', ['label' => ['class' => 'required', 'text' => 'Password'], 'minlength' => $minlength, 'placeholder' => 'Must have atleast ' . $minlength . ' characters', 'class' => 'input radius']) ?>
            </div>
            <div class="medium-12 large-6 cell">
                <?= $this->Html->link(
                'Already have an account? Sign in!',
                ['controller' => 'Users', 'action' => 'login'],
                ['class' => 'button expanded hollow success radius', 'type' => 'button']);
                ?>
            </div>
            <div class="medium-12 large-6 cell">
                <?= $this->Form->button(__('Register'), ['class' => 'button expanded radius']); ?>
            </div>
        </div>
    </fieldset>
</div>