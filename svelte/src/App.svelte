

<script>
  import table from "./table.svelte";
  export let tagValues = ["tag1", "tag2", "tag3"];

  $: selectedTags = new Set(["tag1", "tag2"]);

  function handleClick(event) {
    let tag = event.target.textContent
    if (selectedTags.has(tag)){
      selectedTags.delete(tag)
      selectedTags = selectedTags
      console.log("remove ", tag)
      console.log(selectedTags)
    }
    else{
      selectedTags.add(tag)
      selectedTags = selectedTags
      console.log("insert ", tag)
      console.log(selectedTags)
    }
	}
</script>


<main>
  {#each tagValues as t (t)}
    <div class="tags" on:click={handleClick}>{t}</div>
  {/each}
  <p>{Array.from(selectedTags)}</p>
  <div id="table"></div>
</main>

<style>
  .tags{
    background-color: rgb(55, 144, 221);
    color: white;
    margin: .5em;
    padding: .3em;
    width: fit-content;
  }
	/* main {
		text-align: center;
		padding: 1em;
		max-width: 240px;
		margin: 0 auto;
	}

	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}

	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	} */
</style>