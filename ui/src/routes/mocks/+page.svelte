<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import type { MockRule } from '$lib/types';

  let rules: MockRule[] = $state([]);
  let loading = $state(true);

  async function fetchRules() {
    try {
      const res = await fetch('/api/mocks');
      rules = await res.json();
    } catch (error) {
      console.error('Failed to fetch mock rules:', error);
    } finally {
      loading = false;
    }
  }

  async function toggleRule(id: string) {
    try {
      await fetch(`/api/mocks/${id}/toggle`, { method: 'POST' });
      await fetchRules();
    } catch (error) {
      console.error('Failed to toggle rule:', error);
    }
  }

  async function deleteRule(id: string) {
    if (!confirm('Delete this mock rule?')) return;
    try {
      await fetch(`/api/mocks/${id}`, { method: 'DELETE' });
      await fetchRules();
    } catch (error) {
      console.error('Failed to delete rule:', error);
    }
  }

  async function clearAll() {
    if (!confirm('Delete all mock rules?')) return;
    try {
      await fetch('/api/mocks', { method: 'DELETE' });
      rules = [];
    } catch (error) {
      console.error('Failed to clear rules:', error);
    }
  }

  function getMatchTypeLabel(type: string): string {
    switch (type) {
      case 'exact': return 'Exact Match';
      case 'contains': return 'Contains';
      case 'regex': return 'Regex';
      case 'startswith': return 'Starts With';
      case 'endswith': return 'Ends With';
      default: return type;
    }
  }

  function getStatusColorClass(status: number): string {
    if (status >= 200 && status < 300) return 'bg-green-100 text-green-800';
    if (status >= 300 && status < 400) return 'bg-blue-100 text-blue-800';
    if (status >= 400 && status < 500) return 'bg-yellow-100 text-yellow-800';
    return 'bg-red-100 text-red-800';
  }

  onMount(() => {
    fetchRules();
  });
</script>

<div class="container mx-auto px-4 py-8">
  <div class="flex justify-between items-center mb-6">
    <div>
      <h1 class="text-3xl font-bold">Mock Rules</h1>
      <p class="text-gray-600 mt-1">Intercept requests and return custom responses</p>
    </div>
    <div class="flex gap-4">
      <button
        onclick={fetchRules}
        class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
      >
        Refresh
      </button>
      {#if rules.length > 0}
        <button
          onclick={clearAll}
          class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
        >
          Clear All
        </button>
      {/if}
      <button
        onclick={() => goto('/mocks/new')}
        class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700"
      >
        + New Mock
      </button>
    </div>
  </div>

  {#if loading}
    <div class="text-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
      <p class="mt-4 text-gray-600">Loading mock rules...</p>
    </div>
  {:else if rules.length === 0}
    <div class="text-center py-12 bg-white rounded-lg shadow">
      <div class="text-6xl mb-4">🎭</div>
      <h3 class="text-xl font-semibold mb-2">No Mock Rules</h3>
      <p class="text-gray-600 mb-6">Create your first mock rule to intercept requests</p>
      <button
        onclick={() => goto('/mocks/new')}
        class="px-6 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700"
      >
        Create Mock Rule
      </button>
    </div>
  {:else}
    <div class="space-y-4">
      {#each rules as rule}
        <div class="bg-white rounded-lg shadow p-6 hover:shadow-md transition-shadow">
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-3 mb-3">
                <h3 class="text-xl font-semibold">{rule.name}</h3>
                <span class="px-2 py-1 text-xs font-semibold rounded {getStatusColorClass(rule.response.status)}">
                  {rule.response.status}
                </span>
                {#if rule.method}
                  <span class="px-2 py-1 text-xs font-semibold rounded bg-gray-100 text-gray-800">
                    {rule.method}
                  </span>
                {/if}
                <span class="px-2 py-1 text-xs rounded bg-purple-100 text-purple-800">
                  Priority: {rule.priority}
                </span>
                {#if rule.delay_ms}
                  <span class="px-2 py-1 text-xs rounded bg-orange-100 text-orange-800">
                    Delay: {rule.delay_ms}ms
                  </span>
                {/if}
              </div>

              <div class="space-y-2">
                <div class="flex items-center gap-2 text-sm">
                  <span class="text-gray-600 font-medium">URL Pattern:</span>
                  <code class="px-2 py-1 bg-gray-100 rounded font-mono text-sm">{rule.url_pattern}</code>
                  <span class="text-gray-500">({getMatchTypeLabel(rule.url_match_type)})</span>
                </div>

                {#if rule.response.body}
                  <div class="text-sm">
                    <span class="text-gray-600 font-medium">Response Body:</span>
                    <pre class="mt-1 p-2 bg-gray-50 rounded text-xs overflow-x-auto max-h-32">{rule.response.body.length > 200 ? rule.response.body.substring(0, 200) + '...' : rule.response.body}</pre>
                  </div>
                {/if}

                <div class="text-xs text-gray-500">
                  Created: {new Date(rule.created_at).toLocaleString()}
                </div>
              </div>
            </div>

            <div class="flex items-center gap-2 ml-4">
              <button
                onclick={() => toggleRule(rule.id)}
                class="px-4 py-2 rounded font-medium {rule.enabled ? 'bg-green-100 text-green-700 hover:bg-green-200' : 'bg-gray-100 text-gray-700 hover:bg-gray-200'}"
              >
                {rule.enabled ? 'Enabled' : 'Disabled'}
              </button>
              <button
                onclick={() => goto(`/mocks/${rule.id}`)}
                class="px-4 py-2 bg-blue-100 text-blue-700 rounded hover:bg-blue-200"
              >
                Edit
              </button>
              <button
                onclick={() => deleteRule(rule.id)}
                class="px-4 py-2 bg-red-100 text-red-700 rounded hover:bg-red-200"
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
