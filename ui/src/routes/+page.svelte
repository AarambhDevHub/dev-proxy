<script lang="ts">
  import { onMount } from 'svelte';
  import RequestList from '$lib/components/RequestList.svelte';
  import StatsPanel from '$lib/components/StatsPanel.svelte';
  import FilterPanel from '$lib/components/FilterPanel.svelte';
  import type { RecordedRequest, RecordingStats, FilterOptions } from '$lib/types';

  let recordings: RecordedRequest[] = $state([]);
  let stats: RecordingStats | null = $state(null);
  let loading = $state(true);
  let autoRefresh = $state(true);
  let refreshInterval: number;

  let filters: FilterOptions = $state({
    search: '',
    method: '',
    status: undefined,
    minDuration: undefined,
    maxDuration: undefined
  });

  async function fetchRecordings() {
    try {
      const params = new URLSearchParams();

      if (filters.search) params.append('search', filters.search);
      if (filters.method) params.append('method', filters.method);
      if (filters.status) params.append('status', filters.status.toString());
      if (filters.minDuration) params.append('minDuration', filters.minDuration.toString());
      if (filters.maxDuration) params.append('maxDuration', filters.maxDuration.toString());

      const queryString = params.toString();
      const url = `/api/recordings${queryString ? '?' + queryString : ''}`;

      const res = await fetch(url);
      recordings = await res.json();
    } catch (error) {
      console.error('Failed to fetch recordings:', error);
    } finally {
      loading = false;
    }
  }

  async function fetchStats() {
    try {
      const res = await fetch('/api/stats');
      stats = await res.json();
    } catch (error) {
      console.error('Failed to fetch stats:', error);
    }
  }

  async function clearRecordings() {
    if (!confirm('Clear all recordings?')) return;
    try {
      await fetch('/api/recordings', { method: 'DELETE' });
      recordings = [];
      stats = null;
      await fetchStats();
    } catch (error) {
      console.error('Failed to clear recordings:', error);
    }
  }

  function handleFilterChange() {
    fetchRecordings();
  }

  onMount(() => {
    fetchRecordings();
    fetchStats();

    if (autoRefresh) {
      refreshInterval = setInterval(() => {
        fetchRecordings();
        fetchStats();
      }, 2000);
    }

    return () => {
      if (refreshInterval) clearInterval(refreshInterval);
    };
  });

  $effect(() => {
    if (autoRefresh) {
      refreshInterval = setInterval(() => {
        fetchRecordings();
        fetchStats();
      }, 2000);
    } else {
      if (refreshInterval) clearInterval(refreshInterval);
    }
  });
</script>

<div class="container mx-auto px-4 py-8">
  <div class="flex justify-between items-center mb-6">
    <h1 class="text-3xl font-bold">Traffic Recordings</h1>
    <div class="flex gap-4">
      <label class="flex items-center gap-2 cursor-pointer">
        <input
          type="checkbox"
          bind:checked={autoRefresh}
          class="w-4 h-4"
        />
        <span class="text-sm">Auto-refresh</span>
      </label>
      <button
        onclick={() => { fetchRecordings(); fetchStats(); }}
        class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
      >
        Refresh
      </button>
      <button
        onclick={clearRecordings}
        class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
      >
        Clear All
      </button>
    </div>
  </div>

  {#if stats}
    <StatsPanel {stats} />
  {/if}

  <FilterPanel bind:filters onFilterChange={handleFilterChange} />

  {#if loading}
    <div class="text-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
      <p class="mt-4 text-gray-600">Loading recordings...</p>
    </div>
  {:else if recordings.length === 0}
    <div class="text-center py-12 bg-white rounded-lg shadow">
      <p class="text-gray-600">
        {filters.search || filters.method || filters.status
          ? 'No recordings match your filters.'
          : 'No recordings yet. Send some requests through the proxy!'}
      </p>
    </div>
  {:else}
    <RequestList {recordings} />
  {/if}
</div>
