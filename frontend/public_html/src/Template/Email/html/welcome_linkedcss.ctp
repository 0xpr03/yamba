<div class="row">
    <h1>Verifying your account</h1>
    <p>Hey there. Thank you for registering at Yamba!</p>
</div>
<hr>
<div class="row">
    <p>To help us secure your Yamba account please verify your email-address(<?= $email ?>)</p>
</div>
<div class="row">
    <?= $this->Html->link(
    'Verify your account',
    ['controller' => 'Users', 'action' => 'verify', '_full' => true, $token],
    ['class' => 'button expanded', 'type' => 'button', '_target' => 'blank']);
    ?>
</div>
<hr>
<div class="row">
    <p>Button not working? Paste the following link into your browser:</p>
    <?= $this->Html->link(
    $this->Url->build(['controller' => 'Users', 'action' => 'verify', '_full' => true, $token]),
    ['controller' => 'Users', 'action' => 'verify', '_full' => true, $token],
    ['_target' => 'blank']);
    ?>
</div>
<div class="row">
    <p>You're receiving this email because you recently created a Yamba account. If this wasn’t you, please ignore this email.</p>
</div>