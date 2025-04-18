<script lang="ts">
    import { trackedEffect } from "@src/utils/effects.svelte";
    import { ui } from "openchat-client";
    import { onDestroy, tick, type Snippet } from "svelte";
    import { fade } from "svelte/transition";
    import { rtlStore } from "../../stores/rtl";
    import { tooltipStore } from "../../stores/tooltip";
    import type { Alignment, Position } from "../../utils/alignment";
    import Hoverable from "../Hoverable.svelte";

    interface Props {
        enable?: boolean;
        position?: Position;
        align?: Alignment;
        fill?: boolean;
        gutter?: number;
        longPressed?: boolean;
        children: Snippet;
        popupTemplate: Snippet;
        autoWidth?: boolean;
        textLength?: number;
        longestWord?: number;
        uppercase?: boolean;
    }

    let {
        enable = true,
        position = "top",
        align = "start",
        fill = false,
        gutter = 8,
        longPressed = $bindable(false),
        children,
        popupTemplate,
        autoWidth = false,
        textLength = 100,
        longestWord = 10,
        uppercase = false,
    }: Props = $props();

    let target: Hoverable;
    let tooltipContainer: HTMLElement | undefined = $state();
    let hovering: boolean = $state(false);

    let show = $derived(enable && (hovering || longPressed));
    let maxWidth = $derived(
        autoWidth ? "unset" : calculateMaxWidth(textLength, longestWord, ui.mobileWidth),
    );

    trackedEffect("tooltip", () => {
        if (show) {
            showTooltip();
        } else {
            closeTooltip();
        }
    });

    onDestroy(closeTooltip);

    async function showTooltip(): Promise<void> {
        if (!tooltipContainer) return;

        tooltipStore.show(tooltipContainer);

        await tick();

        const dom = target.getDomElement();
        if (dom !== undefined) {
            tooltipStore.position(dom, position, align, gutter);
        }
    }

    function closeTooltip() {
        tooltipStore.hide();
    }

    function calculateMaxWidth(textLength: number, longestWord: number, mobile: boolean): number {
        const MIN_WIDTH = mobile ? 100 : 140;
        const MAX_WIDTH = mobile ? 250 : 300;

        const CHAR_WIDTH = mobile ? 6 : 7;

        let numChars = textLength + 13;
        return (
            Math.max(
                longestWord * CHAR_WIDTH,
                Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, Math.sqrt(numChars) * CHAR_WIDTH * 2)),
            ) / 16
        );
    }
</script>

<Hoverable {fill} bind:this={target} bind:hovering bind:longPressed enableLongPress>
    {@render children()}
</Hoverable>

<div class="tooltip-blueprint">
    <span class="tooltip" bind:this={tooltipContainer}>
        {#if $tooltipStore === tooltipContainer}
            <div
                transition:fade={{ duration: 100 }}
                class={`tooltip-popup ${position} ${align}`}
                class:rtl={$rtlStore}
                class:uppercase
                style={`max-width: ${maxWidth}rem;`}>
                {@render popupTemplate()}
            </div>
        {/if}
    </span>
</div>

<style lang="scss">
    .tooltip {
        position: absolute;
    }

    .tooltip-blueprint {
        display: none;
    }

    .tooltip-popup {
        background-color: var(--menu-bg);
        border: 1px solid var(--menu-bd);
        color: var(--menu-txt);
        $chevron: 8px;
        $offset: 12px;

        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        position: relative;
        @include z-index("tooltip");
        @include font-size(fs-50);
        width: max-content;
        padding: $sp3;
        border-radius: $sp3;
        pointer-events: none;
        word-wrap: break-word;

        &.uppercase {
            text-transform: uppercase;
        }

        &:after {
            display: block;
            position: absolute;
            background-color: inherit;
            width: $chevron;
            height: $chevron;
            transform: rotate(45deg);
            content: "";
        }

        &.right:after {
            left: -5px;
            border-bottom: 1px solid var(--menu-bd);
            border-left: 1px solid var(--menu-bd);
        }

        &.right.rtl:after {
            left: unset;
            right: -5px;
            border-bottom: none;
            border-left: none;
            border-top: 1px solid var(--menu-bd);
            border-right: 1px solid var(--menu-bd);
        }

        &.left:after {
            right: -5px;
            border-top: 1px solid var(--menu-bd);
            border-right: 1px solid var(--menu-bd);
        }

        &.left.rtl:after {
            right: unset;
            left: -5px;
            border-top: none;
            border-right: none;
            border-bottom: 1px solid var(--menu-bd);
            border-left: 1px solid var(--menu-bd);
        }

        &.bottom:after {
            top: -5px;
            border-top: 1px solid var(--menu-bd);
            border-left: 1px solid var(--menu-bd);
        }

        &.top:after {
            bottom: -5px;
            border-bottom: 1px solid var(--menu-bd);
            border-right: 1px solid var(--menu-bd);
        }

        &.left.start:after,
        &.right.start:after {
            top: $offset;
        }
        &.left.end:after,
        &.right.end:after {
            bottom: $offset;
        }
        &.left.center:after,
        &.right.center:after {
            top: calc(50% - 4px);
        }

        &.top.start:after,
        &.bottom.start:after {
            left: $offset;
        }
        &.top.rtl.start:after,
        &.bottom.rtl.start:after {
            left: unset;
            right: $offset;
        }
        &.top.end:after,
        &.bottom.end:after {
            right: $offset;
        }
        &.top.rtl.end:after,
        &.bottom.rtl.end:after {
            right: unset;
            left: $offset;
        }
    }
</style>
