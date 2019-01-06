<?php
$class = 'message';
if (!empty($params['class'])) {
    $class .= ' ' . $params['class'];
}
if (!isset($params['escape']) || $params['escape'] !== false) {
    $message = h($message);
}
?>
<div data-closable class="callout small <?= h($class) ?>">
    <div class="flash">
        <h5><?= $message ?></h5>
        <button class="close-button" aria-label="dismiss alert" type="button" data-close>
            <span aria-hidden="true">&times;</span>
        </button>
    </div>
</div>
<script>
    setTimeout(function () {
        $('div.flash > button.close-button').click();
    }, 5000);
</script>