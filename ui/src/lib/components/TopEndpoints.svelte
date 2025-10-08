<script lang="ts">
  import type { EndpointStats } from '$lib/types';
  import { formatDuration } from '$lib/types';

  interface Props {
    endpoints: EndpointStats[];
  }

  let { endpoints }: Props = $props();
</script>

<div class="bg-white rounded-lg shadow p-6">
  <h3 class="text-lg font-semibold mb-4">Top 10 Endpoints</h3>

  {#if endpoints.length === 0}
    <p class="text-gray-500 text-center py-8">No endpoints recorded</p>
  {:else}
    <div class="overflow-x-auto">
      <table class="w-full">
        <thead>
          <tr class="border-b border-gray-200">
            <th class="text-left py-3 px-4 font-semibold text-sm text-gray-700">Endpoint</th>
            <th class="text-right py-3 px-4 font-semibold text-sm text-gray-700">Requests</th>
            <th class="text-right py-3 px-4 font-semibold text-sm text-gray-700">Avg Duration</th>
            <th class="text-right py-3 px-4 font-semibold text-sm text-gray-700">Errors</th>
            <th class="text-right py-3 px-4 font-semibold text-sm text-gray-700">Error Rate</th>
          </tr>
        </thead>
        <tbody>
          {#each endpoints as endpoint}
            {@const errorRate = ((endpoint.errors / endpoint.count) * 100).toFixed(1)}
            <tr class="border-b border-gray-100 hover:bg-gray-50">
              <td class="py-3 px-4 font-mono text-sm">{endpoint.endpoint}</td>
              <td class="text-right py-3 px-4 text-sm">{endpoint.count}</td>
              <td class="text-right py-3 px-4 text-sm">{formatDuration(endpoint.avg_duration)}</td>
              <td class="text-right py-3 px-4 text-sm">
                <span class="{endpoint.errors > 0 ? 'text-red-600 font-semibold' : 'text-gray-600'}">
                  {endpoint.errors}
                </span>
              </td>
              <td class="text-right py-3 px-4 text-sm">
                <span class="{parseFloat(errorRate) > 0 ? 'text-red-600 font-semibold' : 'text-green-600'}">
                  {errorRate}%
                </span>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
