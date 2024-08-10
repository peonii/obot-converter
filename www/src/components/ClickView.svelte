<script lang="ts">
    import { ClickType } from '$lib';

    export let click: ClickType;
</script>

{#if click !== ClickType.Skip}
<div class={`py-1 px-3 rounded-lg w-fit text-lg ${
    click === ClickType.Click ? 'bg-red-500/60' : 'bg-green-500/60'
}`}>
    {click === ClickType.Click ? 'Down' : 'Up'}
</div>
{/if}
