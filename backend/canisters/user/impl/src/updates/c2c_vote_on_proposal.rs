use crate::governance_clients::nns::GetBallotsResult;
use crate::guards::caller_is_known_group_or_community_canister;
use crate::{read_state, run_regular_jobs};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use types::{C2CError, CanisterId, NnsNeuronId, ProposalId, SnsNeuronId};
use user_canister::c2c_vote_on_proposal::{Response::*, *};

#[update(guard = "caller_is_known_group_or_community_canister", msgpack = true)]
#[trace]
async fn c2c_vote_on_proposal(args: Args) -> Response {
    run_regular_jobs();

    if args.is_nns {
        nns::vote_on_proposal(args.governance_canister_id, args.proposal_id, args.adopt).await
    } else {
        sns::vote_on_proposal(args.governance_canister_id, args.proposal_id, args.adopt).await
    }
}

mod nns {
    use super::*;
    use nns_governance_canister::types::GovernanceError;

    pub async fn vote_on_proposal(governance_canister_id: CanisterId, proposal_id: ProposalId, adopt: bool) -> Response {
        let ballots = match crate::governance_clients::nns::get_ballots(governance_canister_id, proposal_id).await {
            Ok(r) => match r {
                GetBallotsResult::Success(b) if !b.is_empty() => b,
                GetBallotsResult::Success(_) => return NoEligibleNeurons,
                GetBallotsResult::ProposalNotFound => return ProposalNotFound,
                GetBallotsResult::ProposalNotAcceptingVotes => return ProposalNotAcceptingVotes,
            },
            Err(error) => return InternalError(format!("{error:?}")),
        };

        let vote_futures: Vec<_> = ballots
            .into_iter()
            .filter(|(_, vote)| vote.is_none())
            .map(|(neuron_id, _)| register_vote(governance_canister_id, neuron_id, proposal_id, adopt))
            .collect();

        if let Err(error) = futures::future::try_join_all(vote_futures).await {
            InternalError(format!("{error:?}"))
        } else {
            Success
        }
    }

    async fn register_vote(
        governance_canister_id: CanisterId,
        neuron_id: NnsNeuronId,
        proposal_id: ProposalId,
        adopt: bool,
    ) -> Result<Result<(), GovernanceError>, C2CError> {
        crate::governance_clients::nns::register_vote(governance_canister_id, neuron_id, proposal_id, adopt).await
    }
}

mod sns {
    use super::*;
    use sns_governance_canister::types::GovernanceError;

    pub async fn vote_on_proposal(governance_canister_id: CanisterId, proposal_id: ProposalId, adopt: bool) -> Response {
        let (canister_id, now) = read_state(|state| (state.env.canister_id(), state.env.now()));

        let neuron_ids = match crate::governance_clients::sns::list_neurons(governance_canister_id, 10, canister_id, now).await
        {
            Ok(n) if n.is_empty() => return NoEligibleNeurons,
            Ok(n) => n,
            Err(error) => return InternalError(format!("{error:?}")),
        };

        let vote_futures: Vec<_> = neuron_ids
            .into_iter()
            .map(|neuron_id| register_vote(governance_canister_id, neuron_id, proposal_id, adopt))
            .collect();

        if let Err(error) = futures::future::try_join_all(vote_futures).await {
            InternalError(format!("{error:?}"))
        } else {
            Success
        }
    }

    async fn register_vote(
        governance_canister_id: CanisterId,
        neuron_id: SnsNeuronId,
        proposal_id: ProposalId,
        adopt: bool,
    ) -> Result<Result<(), GovernanceError>, C2CError> {
        crate::governance_clients::sns::register_vote(governance_canister_id, neuron_id, proposal_id, adopt).await
    }
}
