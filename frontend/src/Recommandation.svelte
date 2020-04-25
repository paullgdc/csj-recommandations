<script>
    export let name;
    export let upvoteCount;
</script>

<div class="container">
    <h2>{name}</h2>
    <h3>upvotes: {upvoteCount}</h3>
</div>

<style>
    .container {
        border: 1px solid black;
        padding: 1rem 2rem;
    }
</style>
