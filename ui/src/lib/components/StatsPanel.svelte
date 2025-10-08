<script lang="ts">
  import type { RecordingStats } from '$lib/types';
  import { formatDuration } from '$lib/types';

  interface Props {
    stats: RecordingStats;
  }

  let { stats }: Props = $props();

  const successRate = stats.total > 0
    ? ((stats.success / stats.total) * 100).toFixed(1)
    : 0;
</script>

<div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-8 gap-4 mb-6">
  <div class="bg-white rounded-lg shadow p-4">
    <div class="text-sm text-gray-600 mb-1">Total</div>
    <div class="text-2xl font-bold text-gray-900">{stats.total}</div>
  </div>

  <div class="bg-white rounded-lg shadow p-4">
    <div class="text-sm text-gray-600 mb-1">Success</div>
    <div class="text-2xl font-bold text-green-600">{stats.success}</div>
  </div>

  <div class="bg-white rounded-lg shadow p-4">
    <div class="text-sm text-gray-600 mb-1">Redirects</div>
    <div class="text-2xl font-bold text-blue-600">{stats.redirects}</div>
  </div>

  <div class="bg-white rounded-lg shadow p-4">
    <div class="text-sm text-gray-600 mb-1">4xx Errors</div>
    <div class="text-2xl font-bold text-yellow-600">{stats.client_errors}</div>
  </div>

  <div class="bg-white rounded-lg shadow p-4">
    <div class="text-sm text-gray-600 mb-1">5xx Errors</div>
    <div class="text-2xl font-bold text-red-600">{stats.server_errors}</div>
  </div>

  <div class="bg-white rounded-lg shadow p-4">
    <div class="text-sm text-gray-600 mb-1">Success Rate</div>
    <div class="text-2xl font-bold text-purple-600">{successRate}%</div>
  </div>

  <div class="bg-white rounded-lg shadow p-4">
    <div class="text-sm text-gray-600 mb-1">Avg Time</div>
    <div class="text-2xl font-bold text-indigo-600">{formatDuration(stats.avg_duration_ms)}</div>
  </div>

  <div class="bg-white rounded-lg shadow p-4">
    <div class="text-sm text-gray-600 mb-1">Min / Max</div>
    <div class="text-sm font-semibold text-gray-900">
      {formatDuration(stats.min_duration_ms)} / {formatDuration(stats.max_duration_ms)}
    </div>
  </div>
</div>
