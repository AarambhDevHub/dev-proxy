<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import RequestDetail from '$lib/components/RequestDetail.svelte';
  import type { RecordedRequest } from '$lib/types';

  let recording: RecordedRequest | null = $state(null);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    const id = $page.params.id;

    try {
      const res = await fetch(`/api/recordings/${id}`);
      if (res.ok) {
        recording = await res.json();
      } else {
        error = 'Recording not found';
      }
    } catch (err) {
      error = 'Failed to load recording';
    } finally {
      loading = false;
    }
  });
</script>

<div class="container mx-auto px-4 py-8">
  <div class="mb-6">
    <a href="/" class="text-blue-600 hover:text-blue-700 flex items-center gap-2">
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
      </svg>
      Back to recordings
    </a>
  </div>

  {#if loading}
    <div class="text-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
      <p class="mt-4 text-gray-600">Loading recording...</p>
    </div>
  {:else if error}
    <div class="text-center py-12 bg-white rounded-lg shadow">
      <p class="text-red-600">{error}</p>
    </div>
  {:else if recording}
    <RequestDetail {recording} />
  {/if}
</div>
