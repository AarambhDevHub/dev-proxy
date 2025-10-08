<script lang="ts">
  import type { RecordedRequest } from '$lib/types';
  import { formatBody, formatDuration, getStatusColor } from '$lib/types';

  interface Props {
    recording: RecordedRequest;
  }

  let { recording }: Props = $props();

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

<div class="space-y-6">
  <!-- Summary -->
  <div class="bg-white rounded-lg shadow p-6">
    <div class="flex items-center gap-3 mb-4">
      <span
        class="px-3 py-1 text-sm font-semibold rounded bg-{getMethodColor(recording.method)}-100 text-{getMethodColor(recording.method)}-800"
      >
        {recording.method}
      </span>
      {#if recording.response}
        <span
          class="px-3 py-1 text-sm font-semibold rounded bg-{getStatusColor(recording.response.status)}-100 text-{getStatusColor(recording.response.status)}-800"
        >
          {recording.response.status}
        </span>
      {/if}
      <span class="text-sm text-gray-600">
        {formatDuration(recording.duration_ms)}
      </span>
    </div>
    <p class="font-mono text-lg text-gray-900 break-all mb-2">
      {recording.url}
    </p>
    <p class="text-sm text-gray-500">
      {new Date(recording.timestamp).toLocaleString()}
    </p>
  </div>

  <!-- Request Headers -->
  <div class="bg-white rounded-lg shadow p-6">
    <h3 class="text-lg font-semibold mb-4">Request Headers</h3>
    <div class="space-y-2">
      {#each Object.entries(recording.headers) as [key, value]}
        <div class="flex">
          <span class="font-mono text-sm text-gray-600 w-48">{key}:</span>
          <span class="font-mono text-sm text-gray-900 flex-1">{value}</span>
        </div>
      {/each}
    </div>
  </div>

  <!-- Request Body -->
  {#if recording.body}
    <div class="bg-white rounded-lg shadow p-6">
      <h3 class="text-lg font-semibold mb-4">Request Body</h3>
      <pre class="bg-gray-50 p-4 rounded overflow-x-auto text-sm">{formatBody(recording.body)}</pre>
    </div>
  {/if}

  <!-- Response -->
  {#if recording.response}
    <!-- Response Headers -->
    <div class="bg-white rounded-lg shadow p-6">
      <h3 class="text-lg font-semibold mb-4">Response Headers</h3>
      <div class="space-y-2">
        {#each Object.entries(recording.response.headers) as [key, value]}
          <div class="flex">
            <span class="font-mono text-sm text-gray-600 w-48">{key}:</span>
            <span class="font-mono text-sm text-gray-900 flex-1">{value}</span>
          </div>
        {/each}
      </div>
    </div>

    <!-- Response Body -->
    {#if recording.response.body}
      <div class="bg-white rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold mb-4">Response Body</h3>
        <pre class="bg-gray-50 p-4 rounded overflow-x-auto text-sm">{formatBody(recording.response.body)}</pre>
      </div>
    {/if}
  {/if}
</div>
