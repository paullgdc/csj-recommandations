<script>
  import { createEventDispatcher } from 'svelte';
  export let name;
  export let upvoteCount;
  export let upvoted;

  const dispatch = createEventDispatcher();
</script>

<style>
  :root {
    --upvoted: rgb(69, 221, 69);
    --unvoted: rgb(0, 0, 0);
  }

  .infos {
    padding: 1rem 0rem;
  }
  .container {
    /* padding: 0rem 2rem; */
    border: 1px solid black;
    display: flex;
    /* justify-content: space-between; */
    align-items: center;
  }
  .upvote {
    margin: 0rem 1rem;
    --diameter: 3.2rem;
    width: var(--diameter);
    height: var(--diameter);
    border: 1px solid rgb(0, 0, 0);
    border-radius: calc(var(--diameter) / 2);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: ease-in-out 200ms;
  }

  .upvote:hover {
    transform: scale(1.1);
  }

  .upvoted {
    border-color: var(--upvoted);
    color: var(--upvoted);
  }

  .upvoted:hover {
    transform: scale(0.9);
  }
  
</style>

<div class="container">
  <div class="upvote {upvoted ? "upvoted" : ""}" on:click="{() => dispatch("upvote")}">+1</div>
  <div class="infos">
    <h3>{name}</h3>
    <h5>upvotes: {upvoteCount}</h5>
  </div>
</div>
