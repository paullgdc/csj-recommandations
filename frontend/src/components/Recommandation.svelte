<script>
  import { createEventDispatcher } from 'svelte';
  export let name;
  export let upvoteCount;
  export let upvoted;
  export let isDeletable = false;

  const dispatch = createEventDispatcher();
  function onUpvote() {
    dispatch("upvote");
  }
  function onDelete() {
    dispatch("delete");
  }
</script>

<style>
  :root {
    --upvoted: rgb(69, 221, 69);
    --unvoted: rgb(0, 0, 0);
  }

  .container {
    /* padding: 0rem 2rem; */
    border: 1px solid black;
    padding: 1rem;
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

  .spacer {
    flex: 1;
  }

  .delete {
    visibility: hidden;
    height: 3.2rem;
    width: 3.2rem;
    border-radius: 100%;
    /* border: 1px black solid; */
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 2rem;
    background-color: rgb(231, 49, 49);
    color: white;
    transition: 0.2s;
  }

  .delete:hover {
    background-color: rgb(179, 38, 38);
    transform: scale(1.1);
  }

  .delete-visible {
    visibility: visible;
  }
  
</style>

<div class="container">
  <div class="upvote {upvoted ? "upvoted" : ""}" on:click="{onUpvote}">+1</div>
  <div class="infos">
    <h3>{name}</h3>
    <h5>upvotes: {upvoteCount}</h5>
  </div>
  <div class="spacer"></div>
  <div class="delete { isDeletable ? "delete-visible" : ""}" on:click="{onDelete}">&#x2715</div>
</div>
