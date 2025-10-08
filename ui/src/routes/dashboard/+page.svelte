<script lang="ts">
  import { onMount } from 'svelte';
  import type { Analytics, RecordingStats } from '$lib/types';
  import { formatDuration } from '$lib/types';
  import StatsPanel from '$lib/components/StatsPanel.svelte';
  import MethodChart from '$lib/components/charts/MethodChart.svelte';
  import StatusChart from '$lib/components/charts/StatusChart.svelte';
  import TimelineChart from '$lib/components/charts/TimelineChart.svelte';
  import TopEndpoints from '$lib/components/TopEndpoints.svelte';

  let stats: RecordingStats | null = $state(null);
  let analytics: Analytics | null = $state(null);
  let loading = $state(true);
  let autoRefresh = $state(true);
  let refreshInterval: number;

  async function fetchData() {
    try {
      const [statsRes, analyticsRes] = await Promise.all([
        fetch('/api/stats'),
        fetch('/api/analytics')
      ]);

      stats = await statsRes.json();
      analytics = await analyticsRes.json();
    } catch (error) {
      console.error('Failed to fetch dashboard data:', error);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    fetchData();

    if (autoRefresh) {
      refreshInterval = setInterval(fetchData, 3000);
    }

    return () => {
      if (refreshInterval) clearInterval(refreshInterval);
    };
  });

  $effect(() => {
    if (autoRefresh) {
      refreshInterval = setInterval(fetchData, 3000);
    } else {
      if (refreshInterval) clearInterval(refreshInterval);
    }
  });
</script>

<div class="container mx-auto px-4 py-8">
  <div class="flex justify-between items-center mb-6">
    <h1 class="text-3xl font-bold">Traffic Dashboard</h1>
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
        onclick={fetchData}
        class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
      >
        Refresh
      </button>
    </div>
  </div>

  {#if loading}
    <div class="text-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
      <p class="mt-4 text-gray-600">Loading dashboard...</p>
    </div>
  {:else if stats && analytics}
    <StatsPanel {stats} />

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
      <MethodChart data={analytics.method_distribution} />
      <StatusChart data={analytics.status_distribution} />
    </div>

    <div class="mb-6">
      <TimelineChart data={analytics.timeline} />
    </div>

    <TopEndpoints endpoints={analytics.top_endpoints} />
  {:else}
    <div class="text-center py-12 bg-white rounded-lg shadow">
      <p class="text-gray-600">No data available. Start recording some traffic!</p>
    </div>
  {/if}
</div>
