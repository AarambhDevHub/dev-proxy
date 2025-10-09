<script lang="ts">
  import type { RecordedRequest } from '$lib/types';
  import { formatBody, formatDuration, getStatusColor } from '$lib/types';

  interface Props {
    recording: RecordedRequest;
  }

  let { recording }: Props = $props();

  let replaying = $state(false);
  let replayResult: RecordedRequest | null = $state(null);
  let replayError = $state('');

  async function replayRequest() {
    replaying = true;
    replayError = '';
    replayResult = null;

    try {
      const res = await fetch(`/api/recordings/${recording.id}/replay`, {
        method: 'POST'
      });

      if (res.ok) {
        replayResult = await res.json();
      } else {
        replayError = 'Failed to replay request';
      }
    } catch (error) {
      replayError = 'Network error during replay';
    } finally {
      replaying = false;
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

<div class="space-y-6">
   <!-- Summary -->
  <div class="bg-white rounded-lg shadow p-6">
    <div class="flex items-center justify-between mb-4">
      <div class="flex items-center gap-3">
        <span class="px-3 py-1 text-sm font-semibold rounded bg-{getMethodColor(recording.method)}-100 text-{getMethodColor(recording.method)}-800">
          {recording.method}
        </span>
        {#if recording.response}
          <span class="px-3 py-1 text-sm font-semibold rounded bg-{getStatusColor(recording.response.status)}-100 text-{getStatusColor(recording.response.status)}-800">
            {recording.response.status}
          </span>
        {/if}
        <span class="text-sm text-gray-600">
          {formatDuration(recording.duration_ms)}
        </span>
      </div>

      <!-- Replay Button -->
      <button
        onclick={replayRequest}
        disabled={replaying}
        class="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
      >
        {#if replaying}
          <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
          Replaying...
        {:else}
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          Replay Request
        {/if}
      </button>
    </div>

    <p class="font-mono text-lg text-gray-900 break-all mb-2">{recording.url}</p>
    <p class="text-sm text-gray-500">{new Date(recording.timestamp).toLocaleString()}</p>
  </div>

  <!-- Replay Result -->
  {#if replayError}
    <div class="bg-red-50 border border-red-200 rounded-lg p-4">
      <div class="flex items-center gap-2">
        <svg class="w-5 h-5 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <span class="text-red-800 font-semibold">Replay Failed</span>
      </div>
      <p class="text-red-700 mt-2">{replayError}</p>
    </div>
  {/if}

  {#if replayResult}
    <div class="bg-purple-50 border border-purple-200 rounded-lg p-6">
      <div class="flex items-center gap-2 mb-4">
        <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <h3 class="text-lg font-semibold text-purple-900">Replay Result</h3>
      </div>

      <div class="space-y-3">
        <div class="flex items-center gap-3">
          {#if replayResult.response}
            <span class="px-3 py-1 text-sm font-semibold rounded bg-{getStatusColor(replayResult.response.status)}-100 text-{getStatusColor(replayResult.response.status)}-800">
              Status: {replayResult.response.status}
            </span>
          {/if}
          <span class="text-sm text-gray-700">
            Duration: {formatDuration(replayResult.duration_ms)}
          </span>
        </div>

        {#if replayResult.response?.body}
          <div>
            <h4 class="text-sm font-semibold text-purple-900 mb-2">Response Body:</h4>
            <pre class="bg-white p-4 rounded overflow-x-auto text-sm border border-purple-200">{formatBody(replayResult.response.body)}</pre>
          </div>
        {/if}

        <!-- Comparison -->
        {#if recording.response && replayResult.response}
          <div class="pt-4 border-t border-purple-200">
            <h4 class="text-sm font-semibold text-purple-900 mb-2">Comparison:</h4>
            <div class="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span class="text-gray-600">Original Status:</span>
                <span class="ml-2 font-semibold">{recording.response.status}</span>
              </div>
              <div>
                <span class="text-gray-600">Replay Status:</span>
                <span class="ml-2 font-semibold">{replayResult.response.status}</span>
              </div>
              <div>
                <span class="text-gray-600">Original Duration:</span>
                <span class="ml-2 font-semibold">{formatDuration(recording.duration_ms)}</span>
              </div>
              <div>
                <span class="text-gray-600">Replay Duration:</span>
                <span class="ml-2 font-semibold">{formatDuration(replayResult.duration_ms)}</span>
              </div>
            </div>

            {#if recording.response.status !== replayResult.response.status}
              <div class="mt-3 p-3 bg-yellow-100 border border-yellow-300 rounded text-yellow-800">
                ⚠️ Status code changed from {recording.response.status} to {replayResult.response.status}
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  {/if}

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
