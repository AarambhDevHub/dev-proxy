<script lang="ts">
  import type { TimelinePoint } from '$lib/types';

  interface Props {
    data: TimelinePoint[];
  }

  let { data }: Props = $props();

  const maxDuration = $derived(
    Math.max(...data.map(p => p.duration_ms), 1)
  );

  // Group data into buckets to avoid too many bars
  const bucketedData = $derived(() => {
    if (data.length === 0) return [];

    // If too many points, group them
    if (data.length > 100) {
      const bucketSize = Math.ceil(data.length / 100);
      const buckets: TimelinePoint[] = [];

      for (let i = 0; i < data.length; i += bucketSize) {
        const bucket = data.slice(i, i + bucketSize);
        const avgDuration = bucket.reduce((sum, p) => sum + p.duration_ms, 0) / bucket.length;
        buckets.push({
          ...bucket[0],
          duration_ms: avgDuration
        });
      }

      return buckets;
    }

    return data;
  });

  function formatTime(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  }

  function getStatusColorHex(status: number): string {
    if (status >= 200 && status < 300) return '#10b981'; // green-500
    if (status >= 300 && status < 400) return '#3b82f6'; // blue-500
    if (status >= 400 && status < 500) return '#f59e0b'; // yellow-500
    if (status >= 500) return '#ef4444'; // red-500
    return '#6b7280'; // gray-500
  }

  function getStatusColorHexDark(status: number): string {
    if (status >= 200 && status < 300) return '#059669'; // green-600
    if (status >= 300 && status < 400) return '#2563eb'; // blue-600
    if (status >= 400 && status < 500) return '#d97706'; // yellow-600
    if (status >= 500) return '#dc2626'; // red-600
    return '#4b5563'; // gray-600
  }
</script>

<div class="bg-white rounded-lg shadow p-6">
  <h3 class="text-lg font-semibold mb-4">Request Timeline (Last Hour)</h3>

  {#if data.length === 0}
    <p class="text-gray-500 text-center py-8">No recent requests</p>
  {:else}
    <!-- Chart container with fixed height -->
    <div class="relative bg-gray-50 rounded-lg p-4" style="height: 280px;">
      <div class="w-full h-full flex items-end gap-1 overflow-x-auto">
        {#each bucketedData as point}
          {@const heightPercent = Math.max((point.duration_ms / maxDuration) * 100, 5)}
          {@const heightPx = Math.max((heightPercent / 100) * 240, 10)}
          {@const color = getStatusColorHex(point.status)}
          {@const colorDark = getStatusColorHexDark(point.status)}

          <div
            class="relative group flex-shrink-0 cursor-pointer"
            style="width: 8px;"
          >
            <!-- Bar -->
            <div
              class="rounded-t transition-all duration-200"
              style="height: {heightPx}px; background-color: {color}; width: 100%;"
              onmouseenter={(e) => {
                e.currentTarget.style.backgroundColor = colorDark;
              }}
              onmouseleave={(e) => {
                e.currentTarget.style.backgroundColor = color;
              }}
            ></div>

            <!-- Tooltip -->
            <div class="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 hidden group-hover:block z-10">
              <div class="bg-gray-900 text-white px-3 py-2 rounded-lg shadow-lg text-xs whitespace-nowrap">
                <div class="font-semibold">{point.method} - Status {point.status}</div>
                <div class="text-yellow-300">{point.duration_ms}ms</div>
                <div class="text-gray-300">{formatTime(point.timestamp)}</div>
              </div>
              <!-- Arrow -->
              <div class="absolute top-full left-1/2 transform -translate-x-1/2">
                <div class="w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-900"></div>
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- Stats -->
    <div class="mt-4 flex justify-between text-xs text-gray-600">
      <span class="font-medium">{data.length} requests</span>
      <span class="font-medium">Max duration: {maxDuration}ms</span>
    </div>

    <!-- Legend -->
    <div class="mt-4 flex flex-wrap gap-4 text-xs">
      <div class="flex items-center gap-2">
        <div class="w-3 h-3 rounded" style="background-color: #10b981;"></div>
        <span class="text-gray-600">2xx Success</span>
      </div>
      <div class="flex items-center gap-2">
        <div class="w-3 h-3 rounded" style="background-color: #3b82f6;"></div>
        <span class="text-gray-600">3xx Redirect</span>
      </div>
      <div class="flex items-center gap-2">
        <div class="w-3 h-3 rounded" style="background-color: #f59e0b;"></div>
        <span class="text-gray-600">4xx Client Error</span>
      </div>
      <div class="flex items-center gap-2">
        <div class="w-3 h-3 rounded" style="background-color: #ef4444;"></div>
        <span class="text-gray-600">5xx Server Error</span>
      </div>
    </div>

    <!-- Info note for fast requests -->
    {#if maxDuration < 10}
      <div class="mt-4 p-3 bg-blue-50 border border-blue-200 rounded-lg text-xs text-blue-800">
        <span class="font-semibold">ℹ️ Info:</span> Your requests are very fast (&lt;10ms)! All bars are scaled for visibility - taller bars indicate relatively slower requests.
      </div>
    {/if}
  {/if}
</div>
