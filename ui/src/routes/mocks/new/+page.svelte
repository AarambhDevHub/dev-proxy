<script lang="ts">
  import { goto } from '$app/navigation';
  import type { CreateMockRule } from '$lib/types';

  let rule: CreateMockRule = $state({
    name: '',
    enabled: true,
    priority: 0,
    method: '',
    url_pattern: '',
    url_match_type: 'contains',
    response: {
      status: 200,
      headers: { 'content-type': 'application/json' },
      body: '{"message": "Mocked response"}'
    },
    delay_ms: undefined
  });

  let headersText = $state('content-type: application/json');
  let saving = $state(false);
  let error = $state('');

  async function save() {
    if (!rule.name || !rule.url_pattern) {
      error = 'Name and URL pattern are required';
      return;
    }

    saving = true;
    error = '';

    try {
      // Parse headers
      const headers: Record<string, string> = {};
      headersText.split('\n').forEach(line => {
        const [key, ...valueParts] = line.split(':');
        if (key && valueParts.length > 0) {
          headers[key.trim()] = valueParts.join(':').trim();
        }
      });

      const payload = {
        ...rule,
        method: rule.method || undefined,
        response: {
          ...rule.response,
          headers
        }
      };

      const res = await fetch('/api/mocks', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      });

      if (res.ok) {
        goto('/mocks');
      } else {
        error = 'Failed to create mock rule';
      }
    } catch (e) {
      error = 'Failed to create mock rule: ' + e;
    } finally {
      saving = false;
    }
  }

  function formatJSON() {
    try {
      const parsed = JSON.parse(rule.response!.body);
      rule.response!.body = JSON.stringify(parsed, null, 2);
    } catch {
      // Invalid JSON, keep as is
    }
  }
</script>

<div class="container mx-auto px-4 py-8 max-w-4xl">
  <div class="flex items-center gap-4 mb-6">
    <button
      onclick={() => goto('/mocks')}
      class="text-blue-600 hover:text-blue-700"
    >
      ← Back
    </button>
    <h1 class="text-3xl font-bold">Create Mock Rule</h1>
  </div>

  {#if error}
    <div class="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700">
      {error}
    </div>
  {/if}

  <div class="bg-white rounded-lg shadow p-6 space-y-6">
    <!-- Basic Info -->
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">
          Rule Name *
        </label>
        <input
          type="text"
          bind:value={rule.name}
          placeholder="e.g., Mock User API"
          class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">
          Priority
        </label>
        <input
          type="number"
          bind:value={rule.priority}
          class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <p class="text-xs text-gray-500 mt-1">Higher priority rules match first</p>
      </div>
    </div>

    <!-- Request Matching -->
    <div class="border-t pt-6">
      <h3 class="text-lg font-semibold mb-4">Request Matching</h3>

      <div class="grid grid-cols-2 gap-4 mb-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            HTTP Method (Optional)
          </label>
          <select
            bind:value={rule.method}
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="">Any Method</option>
            <option value="GET">GET</option>
            <option value="POST">POST</option>
            <option value="PUT">PUT</option>
            <option value="PATCH">PATCH</option>
            <option value="DELETE">DELETE</option>
          </select>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Match Type
          </label>
          <select
            bind:value={rule.url_match_type}
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="exact">Exact Match</option>
            <option value="contains">Contains</option>
            <option value="startswith">Starts With</option>
            <option value="endswith">Ends With</option>
            <option value="regex">Regex</option>
          </select>
        </div>
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">
          URL Pattern *
        </label>
        <input
          type="text"
          bind:value={rule.url_pattern}
          placeholder="e.g., /api/users"
          class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>
    </div>

    <!-- Mock Response -->
    <div class="border-t pt-6">
      <h3 class="text-lg font-semibold mb-4">Mock Response</h3>

      <div class="grid grid-cols-2 gap-4 mb-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Status Code
          </label>
          <input
            type="number"
            bind:value={rule.response.status}
            min="100"
            max="599"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Delay (ms)
          </label>
          <input
            type="number"
            bind:value={rule.delay_ms}
            min="0"
            placeholder="No delay"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>

      <div class="mb-4">
        <label class="block text-sm font-medium text-gray-700 mb-2">
          Response Headers (one per line, format: key: value)
        </label>
        <textarea
          bind:value={headersText}
          rows="3"
          class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm"
        ></textarea>
      </div>

      <div>
        <div class="flex justify-between items-center mb-2">
          <label class="block text-sm font-medium text-gray-700">
            Response Body
          </label>
          <button
            onclick={formatJSON}
            class="text-sm text-blue-600 hover:text-blue-700"
          >
            Format JSON
          </button>
        </div>
        <textarea
          bind:value={rule.response.body}
          rows="10"
          class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm"
        ></textarea>
      </div>
    </div>

    <!-- Options -->
    <div class="border-t pt-6">
      <label class="flex items-center gap-2 cursor-pointer">
        <input
          type="checkbox"
          bind:checked={rule.enabled}
          class="w-4 h-4"
        />
        <span class="text-sm font-medium text-gray-700">Enable this rule immediately</span>
      </label>
    </div>

    <!-- Actions -->
    <div class="flex justify-end gap-4 border-t pt-6">
      <button
        onclick={() => goto('/mocks')}
        class="px-6 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
      >
        Cancel
      </button>
      <button
        onclick={save}
        disabled={saving}
        class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
      >
        {saving ? 'Creating...' : 'Create Mock Rule'}
      </button>
    </div>
  </div>
</div>
