<script lang="ts">
    import CommunityCard from "../explore/CommunityCard.svelte";
    import CommunityDetails from "./CommunityDetails.svelte";
    import {
        type OpenChat,
        defaultChatRules,
        selectedCommunity,
        currentCommunityRules,
        currentCommunityReferrals,
    } from "openchat-client";
    import CommunityDetailsHeader from "./CommunityDetailsHeader.svelte";
    import { getContext } from "svelte";

    const client = getContext<OpenChat>("client");

    let frozen = $derived(client.isCommunityFrozen($selectedCommunity?.id));
    let rules = $derived($currentCommunityRules ?? defaultChatRules("community"));
    let canDelete = $derived(
        $selectedCommunity !== undefined &&
            !frozen &&
            client.canDeleteCommunity($selectedCommunity.id),
    );
    let canEdit = $derived(
        $selectedCommunity !== undefined &&
            !frozen &&
            client.canEditCommunity($selectedCommunity.id),
    );
    let canInvite = $derived(
        $selectedCommunity !== undefined && !frozen && client.canInviteUsers($selectedCommunity.id),
    );
</script>

{#if $selectedCommunity}
    <CommunityDetailsHeader community={$selectedCommunity} {canEdit} level={"community"} />
    <div class="body">
        <CommunityCard
            id={$selectedCommunity.id.communityId}
            name={$selectedCommunity.name}
            description={$selectedCommunity.description}
            banner={$selectedCommunity.banner}
            avatar={$selectedCommunity.avatar}
            memberCount={$selectedCommunity.memberCount}
            gateConfig={$selectedCommunity.gateConfig}
            language={$selectedCommunity.primaryLanguage}
            flags={0}
            channelCount={0}
            header
            verified={$selectedCommunity.verified} />
        <CommunityDetails
            {canDelete}
            {canInvite}
            {rules}
            metrics={$selectedCommunity.metrics}
            community={$selectedCommunity}
            referrals={$currentCommunityReferrals} />
    </div>
{/if}

<style lang="scss">
    .body {
        @include nice-scrollbar();
    }
</style>
