<script lang="ts">
  import type { RecordedRequest } from '$lib/types';
  import { formatDuration, getStatusColor } from '$lib/types';

  interface Props {
    recordings: RecordedRequest[];
  }

  let { recordings }: Props = $props();

  async function quickReplay(e: Event, id: string) {
    e.preventDefault();
    e.stopPropagation();

    const confirmed = confirm('Replay this request?');
    if (!confirmed) return;

    try {
      const res = await fetch(`/api/recordings/${id}/replay`, {
        method: 'POST'
      });

      if (res.ok) {
        alert('Request replayed successfully! Check the recording details for results.');
      } else {
        alert('Failed to replay request');
      }
    } catch (error) {
      alert('Network error during replay');
    }
  }


  function getMethodColor(method: string): string {
    switch (method) {
      case 'GET': return 'blue';
      case 'POST': return 'green';
      case 'PUT': return 'yellow';
      case 'DELETE': return 'red';
      case 'PATCH': return 'purple';
      default: return 'gray';
    }
  }
</script>

<div class="space-y-3">
  {#each recordings as recording}
    <a
      href="/recording/{recording.id}"
      class="block bg-white rounded-lg shadow hover:shadow-md transition-shadow p-4"
    >
      <div class="flex items-start justify-between">
        <div class="flex-1">
          <div class="flex items-center gap-3 mb-2">
            <span class="px-2 py-1 text-xs font-semibold rounded bg-{getMethodColor(recording.method)}-100 text-{getMethodColor(recording.method)}-800">
              {recording.method}
            </span>
            {#if recording.response}
              <span class="px-2 py-1 text-xs font-semibold rounded bg-{getStatusColor(recording.response.status)}-100 text-{getStatusColor(recording.response.status)}-800">
                {recording.response.status}
              </span>
            {/if}
            <span class="text-xs text-gray-500">
              {formatDuration(recording.duration_ms)}
            </span>

            <!-- Quick Replay Button -->
            <button
              onclick={(e) => quickReplay(e, recording.id)}
              class="ml-auto px-3 py-1 text-xs bg-purple-100 text-purple-700 rounded hover:bg-purple-200 flex items-center gap-1"
              title="Replay this request"
            >
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              Replay
            </button>
          </div>
          <p class="font-mono text-sm text-gray-900 break-all">{recording.url}</p>
          <p class="text-xs text-gray-500 mt-1">
            {new Date(recording.timestamp).toLocaleString()}
          </p>
        </div>
        <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
        </svg>
      </div>
    </a>
  {/each}
</div>
