import { gql } from "apollo-boost";

export const queries = {
  GET_RECOMMANDATIONS: gql`
  query ($userId: ID!) {
    recommandations {
      id
      name
      link
      media
      upvoteCount
      createdBy
      isUpvotedBy(userId: $userId)
    }
  }
  `,
};

export const mutations = {
  FLIP_UPVOTE: gql`
  mutation ($userId: ID!, $recoId: ID!) {
    flipRecommandationVote(userId: $userId, recoId: $recoId) {
      id
      upvoteCount
      isUpvotedBy(userId: $userId)
    }
  }
  `,
  CREATE_NEW_RECO: gql`
  mutation ($new: NewRecommandation!) {
    createRecommandation(new: $new) {
      id
      name
      link
      media
    }
  }
  `,
  DELETE_RECO: gql`
  mutation($recoId: ID!) {
    deleteRecommandation(recoId: $recoId) {
      id
    }
  }
  `,
};
