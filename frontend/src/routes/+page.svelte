<script lang="ts">
  import { onMount } from "svelte";
  import Counter from "./Counter.svelte";
  import { writable, type Writable } from "svelte/store";

  let balance: Writable<number> = writable(0);
  let counter = 0;

  balance.subscribe(async (value) => {
    counter = value;
  });

  async function fetch_balance() {
    const response = await fetch(`${$api_url}/balance/${$id}`);
    if (response.ok) {
      const data = await response.text();
      balance.set(parseInt(data));
    }
  }

  async function set_balance() {
    await fetch(`${$api_url}/balance/${$id}?balance=${$balance}`, {
      method: "POST",
    });
  }

  let api_url = writable("https://balance-tracker.cofob.dev");
  let id = writable("default");
  onMount(async () => {
    // get api_url and id from local storage
    api_url.set(localStorage.getItem("api_url") || "https://balance-tracker.cofob.dev");
    id.set(localStorage.getItem("id") || "default");
    api_url.subscribe((value) => {
      localStorage.setItem("api_url", value);
    });
    id.subscribe((value) => {
      localStorage.setItem("id", value);
    });

    await fetch_balance();

    balance.subscribe(async (value) => {
      await set_balance();
    });
  });
</script>

<svelte:head>
  <title>Balance</title>
</svelte:head>

<section>
  <input bind:value={$api_url} />
  <input bind:value={$id} />
  <button on:click={fetch_balance}>Fetch Balance</button>
  <Counter bind:count={counter} bind:updated={balance.set} />
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    flex: 0.6;
  }
</style>
