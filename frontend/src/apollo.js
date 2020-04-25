import { gql } from 'apollo-boost';

export const queries = {
  GET_RECOMMANDATIONS: gql`{
    recommandations {
      id
      name
      upvoteCount
    }
  }`,
};
