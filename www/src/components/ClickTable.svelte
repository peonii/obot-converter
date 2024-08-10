
<script lang="ts">
    import { Click, Converter } from '$lib';
	import { onMount, tick } from 'svelte';
	import ClickView from './ClickView.svelte';

    export let converter: Converter

    let currentIdx = 0;
    let pageSize = 20;
    let top = 0;
    let bottom = 0;
    let clickList: HTMLElement;

    let clicks: Click[] = [];
    let rowHeight: number;

    function refreshClicks() {
        clicks = converter.clicks_at_batch(currentIdx, pageSize);
        console.log('Loaded clicks at', currentIdx, pageSize);
    }

    let viewport: HTMLElement;

    // async function refreshTable() {
    //     const { scrollTop } = viewport;


    // }

    async function handleScroll(event: Event) {
        const { scrollTop } = viewport;

        const previousIdx = currentIdx;

        let i = 0;
        let y = 0;

        while (i < converter.length()) {
            if (y + rowHeight > scrollTop) {
                currentIdx = i;
                top = y;
                break;
            }

            y += rowHeight;
            i++;
        }

        while (i < converter.length()) {
            y += rowHeight;
            i++;

            if (y > scrollTop + viewport.offsetHeight) {
                break;
            }
        }

        pageSize = i - currentIdx;

        bottom = (converter.length() - i) * rowHeight;

        refreshClicks();

        console.log(top, bottom, currentIdx, pageSize, y);
    }

    onMount(async () => {
        refreshClicks();
        await tick();
        rowHeight = clickList.querySelector('.click-row')?.clientHeight ?? 0;
    })
</script>

<table bind:this={viewport} id="click-table" on:scroll={handleScroll} class="w-[600px] h-[400px] relative bg-neutral-900 rounded-lg table-fixed border-collapse flex flex-col overflow-y-scroll justify-start scrollbar">
    <thead class="text-white font-bold px-2 text-center sticky w-full z-10 top-0 bg-neutral-900/50 backdrop-blur-md">
        <tr class="flex w-full text-xl border-b border-neutral-700 py-4">
            <th class="w-[30%]">Frame</th>
            <th class="w-[35%]">Player 1</th>
            <th class="w-[35%]">Player 2</th>
        </tr>
    </thead>
    <tbody bind:this={clickList} class="w-full text-xl px-2 flex flex-col" style={
    `
        padding-top: ${top}px;
        padding-bottom: ${bottom}px;
    `
    }>
        {#each clicks as click, i}
            <tr class={`click-row flex text-white text-center w-full min-h-[3.5rem] h-[3.5rem] flex items-center border-b border-neutral-700/30`} style={`
            `}>
                <td class="text-center w-[30%]">{click.frame}</td>
                <td class="text-center w-[35%] flex justify-center"><ClickView click={click.p1} /></td>
                <td class="text-center w-[35%] flex justify-center"><ClickView click={click.p2} /></td>
            </tr>
        {/each}
    </tbody>
</table>


<style>
    .scrollbar::-webkit-scrollbar {
    }

    ::-webkit-scrollbar-track {
        background-color: transparent;
    }

    ::-webkit-scrollbar-thumb {
        background-color: rgba(255, 255, 255, 0.2);
    }
</style>