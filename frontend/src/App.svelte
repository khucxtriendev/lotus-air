<script lang="ts">
  import { onMount } from 'svelte';
  import ConnectionPopup from './lib/components/ConnectionPopup.svelte';
  import ControlFlyout from './lib/components/ControlFlyout.svelte';

  let currentHash = window.location.hash;

  onMount(() => {
    const handleHash = () => {
      currentHash = window.location.hash;
    };
    window.addEventListener('hashchange', handleHash);
    return () => window.removeEventListener('hashchange', handleHash);
  });
</script>

<main>
  {#if currentHash === '#popup'}
    <ConnectionPopup />
  {:else}
    <ControlFlyout />
  {/if}
</main>

<style>
  main {
    width: 100vw;
    height: 100vh;
    overflow: hidden;
  }
</style>
