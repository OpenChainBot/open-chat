<script lang="ts">
    import Home from "./Home.svelte";
    import { identityState, chatsLoading } from "openchat-client";
    import FancyLoader from "../icons/FancyLoader.svelte";
    import LandingPage from "../landingpages/LandingPage.svelte";

    interface Props {
        showLandingPage: boolean;
    }

    let { showLandingPage }: Props = $props();

    let showLoader = $derived(
        $identityState.kind !== "registering" &&
            $identityState.kind !== "challenging" &&
            ($chatsLoading || $identityState.kind === "loading_user"),
    );
</script>

{#if showLandingPage}
    <LandingPage />
{:else if showLoader}
    <div class="loading">
        <div class="inner-loader">
            <FancyLoader />
        </div>
    </div>
{:else}
    <Home on:startVideoCall on:askToSpeak on:hangup />
{/if}

<style lang="scss">
    .loading {
        width: 100vw;
        height: 100vh;
        height: calc(var(--vh, 1vh) * 100);
        height: 100dvh; // firefox will ignore this
        display: grid;
    }

    .inner-loader {
        width: toRem(48);
        height: toRem(48);
        margin: auto;
    }
</style>
