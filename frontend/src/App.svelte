<script>
  import Recommandation from "./Recommandation.svelte";
  import ApolloClient from "apollo-boost";
  import { setClient, query } from "svelte-apollo";
  import { queries } from "./apollo";

  const client = new ApolloClient({ uri: process.env.API_URL });
  setClient(client);

  const recommandations = query(client, {
    query: queries.GET_RECOMMANDATIONS
  });
  
  const user = "@paulostro"
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

  main {
    display: grid;
    grid-template-columns: 10% auto 10%;
    grid-template-rows: auto 2rem 1fr auto;
    grid-template-areas:
      "header header header"
      ". . ."
      ". content ."
      "footer footer footer";
    min-height: 100vh;
  }
</style>

<main>
  <div class="header boxed">
    <h1>CS Japan recommandations</h1>
    <h4>Logged in as {user}</h4>
  </div>

  <div class="content">
    <h2>Recommandations:</h2>
    <div>
      {#await $recommandations}
        <p>Loading...</p>
      {:then res}
        {#each res.data.recommandations as { id, name, upvoteCount }}
          <Recommandation {name} {upvoteCount} />
        {/each}
      {:catch e}
        <p>Error {e}</p>
      {/await}
    </div>
  </div>

  <div class="footer boxed">
    <h2>Made with ❤️ by @paulostro</h2>
  </div>
</main>
