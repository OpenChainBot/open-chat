use crate::{activity_notifications::handle_activity_notification, mutate_state, run_regular_jobs, RuntimeState, TimerJob};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use community_canister::undelete_messages::{Response::*, *};
use group_chat_core::UndeleteMessagesResult;
use std::collections::HashSet;

#[update(msgpack = true)]
#[trace]
fn undelete_messages(args: Args) -> Response {
    run_regular_jobs();

    mutate_state(|state| undelete_messages_impl(args, state))
}

fn undelete_messages_impl(args: Args, state: &mut RuntimeState) -> Response {
    if state.data.is_frozen() {
        return CommunityFrozen;
    }

    let caller = state.env.caller();
    if let Some(member) = state.data.members.get(caller) {
        if member.suspended().value {
            return UserSuspended;
        } else if member.lapsed().value {
            return UserLapsed;
        }

        let now = state.env.now();
        if let Some(channel) = state.data.channels.get_mut(&args.channel_id) {
            match channel
                .chat
                .undelete_messages(member.user_id, args.thread_root_message_index, args.message_ids, now)
            {
                UndeleteMessagesResult::Success(messages) => {
                    if !messages.is_empty() {
                        let message_ids: HashSet<_> = messages.iter().map(|m| m.message_id).collect();
                        state.data.timer_jobs.cancel_jobs(|job| {
                            if let TimerJob::HardDeleteMessageContent(j) = job {
                                j.channel_id == args.channel_id
                                    && j.thread_root_message_index == args.thread_root_message_index
                                    && message_ids.contains(&j.message_id)
                            } else {
                                false
                            }
                        });

                        handle_activity_notification(state);
                    }

                    Success(SuccessResult { messages })
                }
                UndeleteMessagesResult::MessageNotFound => MessageNotFound,
                UndeleteMessagesResult::UserNotInGroup => UserNotInChannel,
                UndeleteMessagesResult::UserSuspended => UserSuspended,
                UndeleteMessagesResult::UserLapsed => UserLapsed,
            }
        } else {
            UserNotInChannel
        }
    } else {
        UserNotInCommunity
    }
}
