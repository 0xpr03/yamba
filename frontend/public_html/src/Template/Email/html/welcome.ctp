<div class="row" style="box-sizing:inherit;margin:0;padding:0">
    <h1 style="box-sizing:inherit;color:inherit;font-family:'Helvetica Neue',Helvetica,Roboto,Arial,sans-serif;font-size:1.5rem;font-style:normal;font-weight:400;line-height:1.4;margin:0;margin-bottom:.5rem;margin-top:0;padding:0;text-rendering:optimizeLegibility">Verifying your account</h1>
    <p style="box-sizing:inherit;font-size:inherit;line-height:1.6;margin:0;margin-bottom:1rem;padding:0;text-rendering:optimizeLegibility">Hey there. Thank you for registering at Yamba!</p>
</div>
<hr style="border-bottom:1px solid #cacaca;border-left:0;border-right:0;border-top:0;box-sizing:content-box;clear:both;height:0;margin:1.25rem auto;max-width:75rem;overflow:visible">
<div class="row" style="box-sizing:inherit;margin:0;padding:0">
    <p style="box-sizing:inherit;font-size:inherit;line-height:1.6;margin:0;margin-bottom:1rem;padding:0;text-rendering:optimizeLegibility">To help us secure your Yamba account please verify your email-address(<?= $email ?>)</p>
</div>
<div class="row" style="box-sizing:inherit;margin:0;padding:0">
    <?= $this->Html->link(
    'Verify your account',
    ['controller' => 'Users', 'action' => 'verify', '_full' => true, $token],
    ['class' => 'button expanded', 'type' => 'button', '_target' => 'blank', 'style' => '-moz-appearance:none;-webkit-appearance:button;-webkit-text-decoration-skip:objects;appearance:none;background-color:#1779ba;border:1px solid transparent;border-radius:0;box-sizing:inherit;color:#fefefe;cursor:pointer;display:block;font-family:inherit;font-size:.9rem;line-height:1;margin:0 0 1rem 0;margin-left:0;margin-right:0;outline:0;padding:.85em 1em;text-align:center;text-decoration:none;transition:background-color .25s ease-out,color .25s ease-out;vertical-align:middle;width:100%']);
    ?>
</div>
<hr style="border-bottom:1px solid #cacaca;border-left:0;border-right:0;border-top:0;box-sizing:content-box;clear:both;height:0;margin:1.25rem auto;max-width:75rem;overflow:visible">
<div class="row" style="box-sizing:inherit;margin:0;padding:0">
    <p style="box-sizing:inherit;font-size:inherit;line-height:1.6;margin:0;margin-bottom:1rem;padding:0;text-rendering:optimizeLegibility">Button not working? Paste the following link into your browser:</p>
    <?= $this->Html->link(
    $this->Url->build(['controller' => 'Users', 'action' => 'verify', '_full' => true, $token]),
    ['controller' => 'Users', 'action' => 'verify', '_full' => true, $token],
    ['_target' => 'blank', 'style' => '-webkit-text-decoration-skip:objects;background-color:transparent;box-sizing:inherit;color:#1779ba;cursor:pointer;line-height:inherit;text-decoration:none']);
    ?>
</div>
<div class="row" style="box-sizing:inherit;margin:0;padding:0">
    <p style="box-sizing:inherit;font-size:inherit;line-height:1.6;margin:0;margin-bottom:1rem;padding:0;text-rendering:optimizeLegibility">You're receiving this email because you recently created a Yamba account. If this wasn’t you, please ignore this email.</p>
</div>