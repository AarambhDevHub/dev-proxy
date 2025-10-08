<script lang="ts">
import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import type { MockRule } from '$lib/types';

  let rule: MockRule | null = $state(null);
  let headersText = $state('');
  let loading = $state(true);
  let saving = $state(false);
  let error = $state('');

  const id = $derived($page.params.id);

  async function fetchRule() {
    try {
      const res = await fetch(`/api/mocks/${id}`);
      if (res.ok) {
        rule = await res.json();
        if (rule) {
          headersText = Object.entries(rule.response.headers)
            .map(([k, v]) => `${k}: ${v}`)
            .join('\n');
        }
      } else {
        error = 'Mock rule not found';
      }
    } catch (e) {
      error = 'Failed to load mock rule';
    } finally {
      loading = false;
    }
  }

  async function save() {
    if (!rule || !rule.name || !rule.url_pattern) {
      error = 'Name and URL pattern are required';
      return;
    }

    saving = true;
    error = '';

    try {
      const headers: Record<string, string> = {};
      headersText.split('\n').forEach(line => {
        const [key, ...valueParts] = line.split(':');
        if (key && valueParts.length > 0) {
          headers[key.trim()] = valueParts.join(':').trim();
        }
      });

      // Create update payload (includes id but not created_at)
      const payload = {
        id: rule.id,
        name: rule.name,
        enabled: rule.enabled,
        priority: rule.priority,
        method: rule.method || undefined,
        url_pattern: rule.url_pattern,
        url_match_type: rule.url_match_type,
        response: {
          status: rule.response.status,
          headers,
          body: rule.response.body
        },
        delay_ms: rule.delay_ms
      };

      const res = await fetch(`/api/mocks/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      });

      if (res.ok) {
        goto('/mocks');
      } else {
        error = 'Failed to update mock rule';
      }
    } catch (e) {
      error = 'Failed to update mock rule: ' + e;
    } finally {
      saving = false;
    }
  }

  function formatJSON() {
    if (!rule) return;
    try {
      const parsed = JSON.parse(rule.response.body);
      rule.response.body = JSON.stringify(parsed, null, 2);
    } catch {
      // Invalid JSON, keep as is
    }
  }

  onMount(() => {
    fetchRule();
  });
</script>

<div class="container mx-auto px-4 py-8 max-w-4xl">
  <div class="flex items-center gap-4 mb-6">
    <button
      onclick={() => goto('/mocks')}
      class="text-blue-600 hover:text-blue-700"
    >
      ← Back
    </button>
    <h1 class="text-3xl font-bold">Edit Mock Rule</h1>
  </div>

  {#if loading}
    <div class="text-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
      <p class="mt-4 text-gray-600">Loading...</p>
    </div>
  {:else if error && !rule}
    <div class="text-center py-12 bg-white rounded-lg shadow">
      <p class="text-red-600">{error}</p>
    </div>
  {:else if rule}
    {#if error}
      <div class="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700">
        {error}
      </div>
    {/if}

    <div class="bg-white rounded-lg shadow p-6 space-y-6">
      <!-- Same form as new page, but with rule data -->
      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Rule Name *
          </label>
          <input
            type="text"
            bind:value={rule.name}
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
        </div>
      </div>

      <div class="border-t pt-6">
        <h3 class="text-lg font-semibold mb-4">Request Matching</h3>

        <div class="grid grid-cols-2 gap-4 mb-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">
              HTTP Method
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
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>

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
              class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
        </div>

        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Response Headers
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

      <div class="border-t pt-6">
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={rule.enabled}
            class="w-4 h-4"
          />
          <span class="text-sm font-medium text-gray-700">Enable this rule</span>
        </label>
      </div>

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
          {saving ? 'Saving...' : 'Save Changes'}
        </button>
      </div>
    </div>
  {/if}
</div>
