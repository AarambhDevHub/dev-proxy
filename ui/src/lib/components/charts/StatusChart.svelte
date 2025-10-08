<script lang="ts">
  interface Props {
    data: Record<string, number>;
  }

  let { data }: Props = $props();

  const chartData = $derived(
    Object.entries(data)
      .map(([status, count]) => ({
        status: parseInt(status),
        count,
        label: getStatusLabel(parseInt(status))
      }))
      .sort((a, b) => b.count - a.count)
  );

  const total = $derived(
    chartData.reduce((sum, item) => sum + item.count, 0)
  );

  const maxCount = $derived(
    Math.max(...chartData.map(item => item.count), 1)
  );

  function getStatusLabel(status: number): string {
    if (status >= 200 && status < 300) return '2xx Success';
    if (status >= 300 && status < 400) return '3xx Redirect';
    if (status >= 400 && status < 500) return '4xx Client Error';
    if (status >= 500) return '5xx Server Error';
    return 'Other';
  }

  function getStatusColorHex(status: number): string {
    if (status >= 200 && status < 300) return '#10b981'; // green-500
    if (status >= 300 && status < 400) return '#3b82f6'; // blue-500
    if (status >= 400 && status < 500) return '#f59e0b'; // yellow-500
    if (status >= 500) return '#ef4444'; // red-500
    return '#6b7280'; // gray-500
  }
</script>

<div class="bg-white rounded-lg shadow p-6">
  <h3 class="text-lg font-semibold mb-4">Status Code Distribution</h3>

  {#if chartData.length === 0}
    <p class="text-gray-500 text-center py-8">No data available</p>
  {:else}
    <div class="space-y-3">
      {#each chartData as item}
        {@const percentage = ((item.count / total) * 100).toFixed(1)}
        {@const barWidth = (item.count / maxCount) * 100}
        {@const color = getStatusColorHex(item.status)}

        <div>
          <div class="flex justify-between items-center mb-1">
            <span class="font-medium text-sm text-gray-700">{item.status} - {item.label}</span>
            <span class="text-sm text-gray-600">{item.count} ({percentage}%)</span>
          </div>
          <div class="w-full bg-gray-200 rounded-full h-3 overflow-hidden">
            <div
              class="h-3 rounded-full transition-all duration-300"
              style="width: {barWidth}%; background-color: {color};"
            ></div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
