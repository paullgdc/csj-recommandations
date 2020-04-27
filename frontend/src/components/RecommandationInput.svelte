<script>
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();

  let reco = {
    name: "",
    type: "",
    link: ""
  };

  let errors = [];

  function validate() {
    errors = [];
    if (reco.name.length === 0) {
      errors.push("Le nom doit etre remplie");
    }

    console.log(errors, reco);
    if (errors.length > 0) {
      return;
    }

    dispatch("confirm", reco);
  }

  function close() {
    dispatch("cancel");
  }
</script>

<style>
  .controls {
    display: flex;
    justify-content: space-between;
    border-top: 1px solid black;
    padding: 1rem 0 0 0;
  }

  .fields {
    display: grid;
  }

  .field:not(:last-child) {
    margin-bottom: 1rem;
  }

  @media (min-width: 600px) {
    .fields {
      grid-template-columns: auto 1fr;
    }
    .fields > :not(:nth-last-child(2)):not(:last-child) {
      margin-bottom: 1rem;
    }
  }

  p.error {
    color: red;
  }

  label {
    padding-right: 1rem;
    align-self: center;
  }
</style>

<div style="display: grid; grid-row-gap: 1rem;">
  <h3>Make a recommandation</h3>
  <div class="fields">

    <label>Nom:</label>
    <input bind:value={reco.name} class="field" />

    <label>Type:</label>
    <select bind:value={reco.type} class="field">
      <option value="MANGA">manga</option>
      <option value="ANIME">anime</option>
      <option value="OTHER">autre</option>
    </select>

    <label>Lien:</label>
    <input bind:value={reco.link} class="field" />
  </div>

  {#if errors.length > 0}
    <div>
      {#each errors as e}
        <p class="error">{e}</p>
      {/each}
    </div>
  {/if}

  <div class="controls">
    <button on:click={validate}>confirm</button>
    <button on:click={close}>cancel</button>
  </div>
</div>
