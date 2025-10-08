<script lang="ts">
  import type { FilterOptions } from '$lib/types';

  interface Props {
    filters: FilterOptions;
    onFilterChange: () => void;
  }

  let { filters, onFilterChange }: Props = $props();

  let showAdvanced = $state(false);

  function handleSearchInput(e: Event) {
    const target = e.target as HTMLInputElement;
    filters.search = target.value;
    onFilterChange();
  }

  function handleMethodChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    filters.method = target.value;
    onFilterChange();
  }

  function handleStatusChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    filters.status = target.value ? parseInt(target.value) : undefined;
    onFilterChange();
  }

  function clearFilters() {
    filters.search = '';
    filters.method = '';
    filters.status = undefined;
    filters.minDuration = undefined;
    filters.maxDuration = undefined;
    onFilterChange();
  }

  const hasActiveFilters = $derived(
    filters.search ||
    filters.method ||
    filters.status ||
    filters.minDuration ||
    filters.maxDuration
  );
</script>

<div class="bg-white rounded-lg shadow p-4 mb-6">
  <div class="flex items-center gap-4 mb-4">
    <div class="flex-1">
      <input
        type="text"
        placeholder="Search in URLs, methods, and bodies..."
        value={filters.search}
        oninput={handleSearchInput}
        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
    </div>

    <select
      value={filters.method}
      onchange={handleMethodChange}
      class="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
    >
      <option value="">All Methods</option>
      <option value="GET">GET</option>
      <option value="POST">POST</option>
      <option value="PUT">PUT</option>
      <option value="PATCH">PATCH</option>
      <option value="DELETE">DELETE</option>
      <option value="OPTIONS">OPTIONS</option>
      <option value="HEAD">HEAD</option>
    </select>

    <select
      value={filters.status || ''}
      onchange={handleStatusChange}
      class="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
    >
      <option value="">All Status</option>
      <option value="200">200 OK</option>
      <option value="201">201 Created</option>
      <option value="204">204 No Content</option>
      <option value="301">301 Moved</option>
      <option value="302">302 Found</option>
      <option value="400">400 Bad Request</option>
      <option value="401">401 Unauthorized</option>
      <option value="403">403 Forbidden</option>
      <option value="404">404 Not Found</option>
      <option value="500">500 Server Error</option>
      <option value="502">502 Bad Gateway</option>
      <option value="503">503 Unavailable</option>
    </select>

    <button
      onclick={() => showAdvanced = !showAdvanced}
      class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
    >
      {showAdvanced ? '▲' : '▼'} Advanced
    </button>

    {#if hasActiveFilters}
      <button
        onclick={clearFilters}
        class="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700"
      >
        Clear
      </button>
    {/if}
  </div>

  {#if showAdvanced}
    <div class="grid grid-cols-2 gap-4 pt-4 border-t">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">
          Min Duration (ms)
        </label>
        <input
          type="number"
          bind:value={filters.minDuration}
          onchange={onFilterChange}
          placeholder="e.g. 100"
          class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">
          Max Duration (ms)
        </label>
        <input
          type="number"
          bind:value={filters.maxDuration}
          onchange={onFilterChange}
          placeholder="e.g. 5000"
          class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>
    </div>
  {/if}
</div>
