<script>
  import Recommandation from "./components/Recommandation.svelte";
  import Modal from "./components/Modal.svelte";
  import RecommandationInput from "./components/RecommandationInput.svelte";
  import { getRecommandations, handleConfirmReco, handleUpvote, deleteReco } from "./api";

  import ApolloClient from "apollo-boost";
  import { setClient } from "svelte-apollo";

  const client = new ApolloClient({ uri: process.env.API_URL });
  setClient(client);

  const user = "paul";

  const recommandations = getRecommandations(client, user);

  let showCreateReco = false;
</script>

<style>
  .boxed {
    border: 1px solid rgb(12, 98, 226);
    padding: 1rem 1rem;
  }
  .header {
    grid-area: header;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .footer {
    grid-area: footer;
  }

  .content {
    grid-area: content;
  }

  .title {
    visibility: hidden;
    width: 0;
    height: 0;
  }

  @media (min-width: 600px) {
    .title {
      visibility: visible;
      width: inherit;
      height: inherit;
    }
  }

  .reco-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .create-reco-overlay {
    background-color: white;
    width: 90%;
    padding: 2rem;
    border-radius: 1rem;
  }

  @media (min-width: 600px) {
    .create-reco-overlay {
      width: 60%;
    }
  }

  main {
    display: grid;
    grid-template-columns: 10% 1fr 10%;
    grid-template-rows: auto 2rem 1fr 2rem auto;
    grid-template-areas:
      "header header header"
      ". . ."
      ". content ."
      ". . ."
      "footer footer footer";
    min-height: 100vh;
    max-width: 100vw;
  }
</style>

<main>
  <div class="header boxed">
    <h2 class="title">CS Japan recommandations</h2>
    <h4>Logged in as {user}</h4>
  </div>

  <div class="content">
    <div class="reco-header">
      <h2>Recommandations:</h2>
      <button
        on:click={() => {
          showCreateReco = true;
        }}>
        + Make a recommandation
      </button>
    </div>
    <div>
      {#await $recommandations}
        <p>Loading...</p>
      {:then res}
        {#each res.data.recommandations as reco}
          <Recommandation
            name={reco.name}
            upvoteCount={reco.upvoteCount}
            upvoted={reco.isUpvotedBy}
            isDeletable={user === reco.createdBy}
            on:upvote={() => handleUpvote(client, user, reco)}
            on:delete={() => deleteReco(client, user, reco.id)} />
        {/each}
      {:catch e}
        <p>Error {e}</p>
      {/await}
    </div>
  </div>

  <Modal isVisible={showCreateReco}>
    <div class="create-reco-overlay">
      <RecommandationInput
        on:cancel={() => {
          showCreateReco = false;
        }}
        on:confirm={({ detail: reco }) => {
          showCreateReco = false;
          handleConfirmReco(client, user, reco);
        }} />
    </div>
  </Modal>

  <div class="footer boxed">
    <h2>Made with ❤️</h2>
  </div>
</main>
