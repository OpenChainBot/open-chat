use crate::guards::caller_is_owner;
use crate::{mutate_state, read_state, run_regular_jobs, RuntimeState, COMMUNITY_CREATION_LIMIT};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use group_index_canister::c2c_create_community;
use oc_error_codes::OCErrorCode;
use std::collections::HashSet;
use tracing::error;
use types::{CanisterId, CommunityId};
use user_canister::create_community::{Response::*, *};
use utils::document::{validate_avatar, validate_banner};
use utils::text_validation::{
    validate_channel_name, validate_community_name, validate_description, validate_rules, NameValidationError,
    RulesValidationError,
};

#[update(guard = "caller_is_owner", msgpack = true)]
#[trace]
async fn create_community(mut args: Args) -> Response {
    run_regular_jobs();

    args.name = args.name.trim().to_string();
    args.description = args.description.trim().to_string();
    args.rules.text = args.rules.text.trim().to_string();
    args.default_channels = args.default_channels.into_iter().map(|c| c.trim().to_string()).collect();

    let prepare_result = match read_state(|state| prepare(args, state)) {
        Ok(ok) => ok,
        Err(response) => return response,
    };

    match group_index_canister_c2c_client::c2c_create_community(
        prepare_result.group_index_canister_id,
        &prepare_result.create_community_args,
    )
    .await
    {
        Ok(response) => match response {
            c2c_create_community::Response::Success(r) => {
                mutate_state(|state| commit(r.community_id, r.local_user_index_canister_id, state));
                Success(SuccessResult {
                    community_id: r.community_id,
                })
            }
            c2c_create_community::Response::NameTaken => Error(OCErrorCode::NameTaken.into()),
            c2c_create_community::Response::UserNotFound => Error(OCErrorCode::InitiatorNotFound.into()),
            c2c_create_community::Response::InternalError(error) => Error(OCErrorCode::Unknown.with_message(error)),
            c2c_create_community::Response::Error(error) => Error(error),
        },
        Err(error) => {
            error!(?error, "Error calling create community");
            InternalError(format!("{error:?}"))
        }
    }
}

struct PrepareResult {
    group_index_canister_id: CanisterId,
    create_community_args: c2c_create_community::Args,
}

fn prepare(args: Args, state: &RuntimeState) -> Result<PrepareResult, Response> {
    fn is_throttled() -> bool {
        // TODO check here that the user hasn't created too many communities in succession
        false
    }

    let now = state.env.now();
    let is_diamond_member = state.data.is_diamond_member(now);

    if state.data.suspended.value {
        Err(UserSuspended)
    } else if !is_diamond_member {
        Err(Unauthorized)
    } else if state.data.communities.communities_created() >= COMMUNITY_CREATION_LIMIT {
        Err(MaxCommunitiesCreated(COMMUNITY_CREATION_LIMIT))
    } else if is_throttled() {
        Err(Throttled)
    } else if let Err(error) = validate_community_name(&args.name, args.is_public) {
        Err(match error {
            NameValidationError::TooShort(s) => NameTooShort(s),
            NameValidationError::TooLong(l) => NameTooLong(l),
            NameValidationError::Reserved => NameReserved,
        })
    } else if let Err(error) = validate_description(&args.description) {
        Err(DescriptionTooLong(error))
    } else if let Err(error) = validate_rules(args.rules.enabled, &args.rules.text) {
        return Err(match error {
            RulesValidationError::TooShort(s) => RulesTooShort(s),
            RulesValidationError::TooLong(l) => RulesTooLong(l),
        });
    } else if let Err(error) = validate_avatar(args.avatar.as_ref()) {
        Err(AvatarTooBig(error))
    } else if let Err(error) = validate_banner(args.banner.as_ref()) {
        Err(BannerTooBig(error))
    } else if args
        .gate_config
        .as_ref()
        .map(|g| !g.validate(state.data.test_mode))
        .unwrap_or_default()
    {
        Err(AccessGateInvalid)
    } else if !default_channels_valid(&args.default_channels) {
        Err(DefaultChannelsInvalid)
    } else {
        let create_community_args = c2c_create_community::Args {
            is_public: args.is_public,
            name: args.name,
            description: args.description,
            rules: args.rules,
            history_visible_to_new_joiners: args.history_visible_to_new_joiners,
            avatar: args.avatar,
            banner: args.banner,
            permissions: args.permissions,
            gate_config: args.gate_config,
            default_channels: args.default_channels,
            default_channel_rules: args.default_channel_rules,
            primary_language: args.primary_language,
        };
        Ok(PrepareResult {
            group_index_canister_id: state.data.group_index_canister_id,
            create_community_args,
        })
    }
}

fn default_channels_valid(default_channels: &[String]) -> bool {
    if default_channels.is_empty() || default_channels.iter().any(|channel| validate_channel_name(channel).is_err()) {
        return false;
    }

    let names: HashSet<String> = default_channels.iter().map(|name| name.to_lowercase()).collect();

    if names.len() != default_channels.len() {
        return false;
    }

    true
}

fn commit(community_id: CommunityId, local_user_index_canister_id: CanisterId, state: &mut RuntimeState) {
    let now = state.env.now();
    state.data.communities.create(community_id, local_user_index_canister_id, now);
}
