import { query, mutate } from "svelte-apollo";
import { queries, mutations } from "./apollo";

// Helpers

function updateRecoCache(cache, user, updateFn) {
  const { recommandations } = cache.readQuery({
    query: queries.GET_RECOMMANDATIONS,
    variables: { userId: user }
  });
  const newRecommandations = updateFn(recommandations);
  cache.writeQuery({
    query: queries.GET_RECOMMANDATIONS,
    data: { recommandations: newRecommandations },
    variables: { userId: user }
  });
}

// Query

export function getRecommandations(client, user) {
  return query(client, {
    query: queries.GET_RECOMMANDATIONS,
    variables: { userId: user }
  });
}

// Mutate

export function handleUpvote(client, user, reco) {
  return mutate(client, {
    mutation: mutations.FLIP_UPVOTE,
    variables: {
      userId: user,
      recoId: reco.id
    },
    optimisticResponse: {
      flipRecommandationVote: {
        ...reco,
        upvoteCount: reco.upvoteCount + (!reco.isUpvotedBy * 2 - 1),
        isUpvotedBy: !reco.isUpvotedBy,
      }
    }
  });
}

export function handleConfirmReco(client, user, newReco) {
  return mutate(client, {
    mutation: mutations.CREATE_NEW_RECO,
    variables: {
      new: newReco
    },
    update: (cache, { data: { createRecommandation } }) => {
      updateRecoCache(cache, user, (recommandations) => {
        return [
          ...recommandations,
          { ...createRecommandation, upvoteCount: 0, isUpvotedBy: false, createdBy: user },
        ];
      });
    }
  });
}

export function deleteReco(client, user, recoId) {
  return mutate(client, {
    mutation: mutations.DELETE_RECO,
    variables: { recoId },
    optimisticResponse: {
      deleteRecommandation: {
        id: recoId, __typename: "Recommandation",
      }
    },
    update(cache, { data: { deleteRecommandation } }) {
      updateRecoCache(cache, user, (recommandations) => {
        return recommandations.filter(reco => reco.id !== deleteRecommandation.id);
      });
    }
  });
}
