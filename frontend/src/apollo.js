import { gql } from 'apollo-boost';

export const queries = {
  GET_RECOMMANDATIONS: gql`
  query ($user: String!) {
    recommandations {
      id
      name
      link
      media
      upvoteCount
      isUpvotedBy(user: $user)
    }
  }
  `,
};

export const mutations = {
  FLIP_UPVOTE: gql`
  mutation ($user: String!, $recoId: ID!) {
    flipRecommandationVote(user: $user, id: $recoId) {
      id
      upvoteCount
      isUpvotedBy(user: $user)
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
};
