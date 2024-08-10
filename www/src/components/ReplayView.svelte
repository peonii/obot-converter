<script lang="ts">
    import { Converter, Click, ClickType, formats, Format, GameVersion } from '$lib';
	import ClickTable from './ClickTable.svelte';

    export let converter: Converter;
    export let replayData: {
        fps: number;
        format: number;
        length: number;
    }
    export let replayName: string;

    $: replayFormatPretty = formats[(replayData?.format ?? 0) as keyof typeof formats][0];

    let selectedSaveFormat = Format.PlainText;

    function saveReplay() {
        const data = converter.save(selectedSaveFormat);

        const blob = new Blob([data], {type: 'application/octet-stream'});
        const url = URL.createObjectURL(blob);

        const link = document.createElement('a');
        link.href = url;
        link.download = `${replayName}.${formats[selectedSaveFormat as keyof typeof formats][1]}`;
        link.click();

        URL.revokeObjectURL(url);

        document.body.removeChild(link);
    }

    function allowedFormats() {
        if (formats[replayData.format][2] === GameVersion.Any) {
            return formats;
        }

        // Filter out formats that are not compatible with the game version
        const entries = Object.entries(formats);
        const allowed = entries.filter(([idx, [format, pretty, version]]) => {
            return version === formats[replayData.format][2] || version === GameVersion.Any;
        })

        return Object.fromEntries(allowed);
    }
</script>


<div class="flex gap-4">
    <ClickTable converter={converter} />
    <div class="flex flex-col gap-2 text-xl">
        <div class="flex gap-2">
            <span class="text-neutral-400 font">Format</span>
            <span class="text-white font-bold">{replayFormatPretty}</span>
        </div>
        <div class="flex gap-2">
            <span class="text-neutral-400 font">FPS</span>
            <span class="text-white font-bold">{replayData?.fps}</span>
        </div>
        <div class="flex gap-2">
            <span class="text-neutral-400 font">Inputs</span>
            <span class="text-white font-bold">{replayData?.length}</span>
        </div>
        <div class="h-max flex-grow"></div>
        <div class="flex gap-4">
            <button on:click={saveReplay} class="bg-neutral-800 text-white rounded-md px-6 py-2 font-medium hover:bg-neutral-700">Save</button>
            <select bind:value={selectedSaveFormat} class="bg-neutral-800 text-white rounded-md px-6 py-2 font-medium hover:bg-neutral-700">
                {#each Object.entries(allowedFormats()) as [format, pretty]}
                    <option value={format}>{pretty[0]}</option>
                {/each}
            </select>
        </div>
    </div>
</div>