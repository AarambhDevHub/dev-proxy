<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import type { ModifierRule } from '$lib/types';

	let rules: ModifierRule[] = $state([]);
	let loading = $state(true);

	async function fetchRules() {
		try {
			const res = await fetch('/api/modifiers');
			rules = await res.json();
		} catch (error) {
			console.error('Failed to fetch modifier rules:', error);
		} finally {
			loading = false;
		}
	}

	async function toggleRule(id: string) {
		try {
			await fetch(`/api/modifiers/${id}/toggle`, { method: 'POST' });
			await fetchRules();
		} catch (error) {
			console.error('Failed to toggle rule:', error);
		}
	}

	async function deleteRule(id: string) {
		if (!confirm('Delete this modifier rule?')) return;

		try {
			await fetch(`/api/modifiers/${id}`, { method: 'DELETE' });
			await fetchRules();
		} catch (error) {
			console.error('Failed to delete rule:', error);
		}
	}

	async function clearAll() {
		if (!confirm('Delete all modifier rules?')) return;

		try {
			await fetch('/api/modifiers', { method: 'DELETE' });
			rules = [];
		} catch (error) {
			console.error('Failed to clear rules:', error);
		}
	}

	function getMatchTypeLabel(type: string): string {
		switch (type) {
			case 'exact':
				return 'Exact Match';
			case 'contains':
				return 'Contains';
			case 'regex':
				return 'Regex';
			case 'startswith':
				return 'Starts With';
			case 'endswith':
				return 'Ends With';
			default:
				return type;
		}
	}

	function getModificationTypeLabel(type: string): string {
		switch (type) {
			case 'replace_body':
				return 'Replace Body';
			case 'add_header':
				return 'Add Header';
			case 'remove_header':
				return 'Remove Header';
			case 'change_status':
				return 'Change Status';
			case 'inject_delay':
				return 'Inject Delay';
			case 'modify_json':
				return 'Modify JSON';
			default:
				return type;
		}
	}

	onMount(() => {
		fetchRules();
	});
</script>

<div class="container mx-auto px-4 py-8">
	<div class="flex justify-between items-center mb-6">
		<h1 class="text-3xl font-bold">Response Modifiers (MITM)</h1>
		<div class="flex gap-4">
			<button
				onclick={() => fetchRules()}
				class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
			>
				Refresh
			</button>
			<button
				onclick={() => goto('/modifiers/new')}
				class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700"
			>
				+ New Modifier
			</button>
			{#if rules.length > 0}
				<button
					onclick={clearAll}
					class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
				>
					Clear All
				</button>
			{/if}
		</div>
	</div>

	{#if loading}
		<div class="text-center py-12">
			<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
			<p class="mt-4 text-gray-600">Loading modifiers...</p>
		</div>
	{:else if rules.length === 0}
		<div class="text-center py-12 bg-white rounded-lg shadow">
			<svg
				class="w-16 h-16 text-gray-400 mx-auto mb-4"
				fill="none"
				stroke="currentColor"
				viewBox="0 0 24 24"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"
				/>
			</svg>
			<p class="text-gray-600 mb-4">No modifier rules configured.</p>
			<button
				onclick={() => goto('/modifiers/new')}
				class="px-6 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700"
			>
				Create Your First Modifier
			</button>
		</div>
	{:else}
		<div class="space-y-4">
			{#each rules as rule}
				<div class="bg-white rounded-lg shadow p-6">
					<div class="flex items-start justify-between">
						<div class="flex-1">
							<div class="flex items-center gap-3 mb-3">
								<h3 class="text-xl font-semibold text-gray-900">{rule.name}</h3>
								<span
									class="px-3 py-1 text-xs font-semibold rounded-full {rule.enabled
										? 'bg-green-100 text-green-800'
										: 'bg-gray-100 text-gray-800'}"
								>
									{rule.enabled ? 'Enabled' : 'Disabled'}
								</span>
								<span class="px-3 py-1 text-xs bg-blue-100 text-blue-800 rounded-full">
									Priority: {rule.priority}
								</span>
							</div>

							<div class="space-y-2 text-sm text-gray-600">
								<div class="flex items-center gap-2">
									<span class="font-medium">Match:</span>
									{#if rule.match_request.method}
										<span class="px-2 py-1 bg-gray-100 rounded">{rule.match_request.method}</span>
									{/if}
									<span class="px-2 py-1 bg-gray-100 rounded font-mono">
										{rule.match_request.url_pattern}
									</span>
									<span class="text-xs text-gray-500">
										({getMatchTypeLabel(rule.match_request.url_match_type)})
									</span>
								</div>

								{#if rule.match_request.status_codes && rule.match_request.status_codes.length > 0}
									<div class="flex items-center gap-2">
										<span class="font-medium">Status Codes:</span>
										{#each rule.match_request.status_codes as status}
											<span class="px-2 py-1 bg-gray-100 rounded">{status}</span>
										{/each}
									</div>
								{/if}

								<div class="flex items-center gap-2 flex-wrap">
									<span class="font-medium">Modifications:</span>
									{#each rule.modifications as mod}
										<span class="px-2 py-1 bg-purple-100 text-purple-700 rounded text-xs">
											{getModificationTypeLabel(mod.type)}
										</span>
									{/each}
								</div>

								<div class="text-xs text-gray-400">
									Created: {new Date(rule.created_at).toLocaleString()}
								</div>
							</div>
						</div>

						<div class="flex gap-2 ml-4">
							<button
								onclick={() => toggleRule(rule.id)}
								class="px-3 py-2 text-sm {rule.enabled
									? 'bg-yellow-100 text-yellow-700 hover:bg-yellow-200'
									: 'bg-green-100 text-green-700 hover:bg-green-200'} rounded"
							>
								{rule.enabled ? 'Disable' : 'Enable'}
							</button>
							<button
								onclick={() => goto(`/modifiers/${rule.id}`)}
								class="px-3 py-2 text-sm bg-blue-100 text-blue-700 rounded hover:bg-blue-200"
							>
								Edit
							</button>
							<button
								onclick={() => deleteRule(rule.id)}
								class="px-3 py-2 text-sm bg-red-100 text-red-700 rounded hover:bg-red-200"
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
