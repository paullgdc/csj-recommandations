import { gql } from 'apollo-boost';

export const queries = {
  GET_RECOMMANDATIONS: gql`
  query ($user: String!) {
    recommandations {
      id
      name
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
};
