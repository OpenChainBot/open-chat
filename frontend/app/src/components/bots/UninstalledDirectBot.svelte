<script lang="ts">
    import page from "page";
    import {
        chatIdentifiersEqual,
        externalBots,
        OpenChat,
        chatListScopeStore as chatListScope,
        type DirectChatIdentifier,
        currentUser,
        installedDirectBots,
    } from "openchat-client";
    import { getContext, tick } from "svelte";
    import { pathParams, routeForScope } from "../../routes";
    import BotInstaller from "./install/BotInstaller.svelte";

    const client = getContext<OpenChat>("client");

    interface Props {
        chatId: DirectChatIdentifier;
        botId: string;
        onClose: () => void;
    }

    let { botId, chatId, onClose }: Props = $props();

    let bot = $derived($externalBots.get(botId));

    function closeInstaller(installed: boolean) {
        if (!installed) {
            if (
                $pathParams.kind === "global_chat_selected_route" &&
                chatIdentifiersEqual(chatId, $pathParams.chatId)
            ) {
                page(routeForScope($chatListScope));
            }
            tick().then(() => client.removeChat(chatId));
        }
        onClose();
    }
</script>

{#if bot !== undefined}
    <BotInstaller
        level={"group"}
        location={{ kind: "direct_chat", userId: $currentUser.userId }}
        {bot}
        onClose={closeInstaller}
        installedBots={$installedDirectBots} />
{/if}
