<script lang="ts">
	import { goto } from '$app/navigation';
	import type { CreateModifierRule, Modification } from '$lib/types';

	let rule: CreateModifierRule = $state({
		name: '',
		enabled: true,
		priority: 0,
		match_request: {
			method: undefined,
			url_pattern: '',
			url_match_type: 'contains',
			status_codes: undefined
		},
		modifications: []
	});

	let statusCodesText = $state('');
	let saving = $state(false);
	let error = $state('');

	// Current modification being added
	let currentModType = $state<string>('replace_body');
	let modConfig = $state<any>({
		pattern: '',
		replacement: '',
		use_regex: false,
		name: '',
		value: '',
		status: 200,
		delay_ms: 100,
		path: '',
		json_value: '{}'
	});

	async function save() {
		if (!rule.name || !rule.match_request.url_pattern) {
			error = 'Name and URL pattern are required';
			return;
		}

		if (rule.modifications.length === 0) {
			error = 'At least one modification is required';
			return;
		}

		saving = true;
		error = '';

		try {
			// Parse status codes
			if (statusCodesText.trim()) {
				const codes = statusCodesText
					.split(',')
					.map((s) => parseInt(s.trim()))
					.filter((n) => !isNaN(n));
				if (codes.length > 0) {
					rule.match_request.status_codes = codes;
				}
			}

			// Clean up method
			if (rule.match_request.method === '') {
				rule.match_request.method = undefined;
			}

			const res = await fetch('/api/modifiers', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(rule)
			});

			if (res.ok) {
				goto('/modifiers');
			} else {
				error = 'Failed to create modifier rule';
			}
		} catch (e) {
			error = `Failed to create modifier rule: ${e}`;
		} finally {
			saving = false;
		}
	}

	function addModification() {
		let modification: Modification;

		switch (currentModType) {
			case 'replace_body':
				modification = {
					type: 'replace_body',
					pattern: modConfig.pattern,
					replacement: modConfig.replacement,
					use_regex: modConfig.use_regex
				};
				break;
			case 'add_header':
				modification = {
					type: 'add_header',
					name: modConfig.name,
					value: modConfig.value
				};
				break;
			case 'remove_header':
				modification = {
					type: 'remove_header',
					name: modConfig.name
				};
				break;
			case 'change_status':
				modification = {
					type: 'change_status',
					status: parseInt(modConfig.status)
				};
				break;
			case 'inject_delay':
				modification = {
					type: 'inject_delay',
					delay_ms: parseInt(modConfig.delay_ms)
				};
				break;
			case 'modify_json':
				try {
					const jsonValue = JSON.parse(modConfig.json_value);
					modification = {
						type: 'modify_json',
						path: modConfig.path,
						value: jsonValue
					};
				} catch {
					error = 'Invalid JSON value';
					return;
				}
				break;
			default:
				return;
		}

		rule.modifications.push(modification);
		// Reset config
		modConfig = {
			pattern: '',
			replacement: '',
			use_regex: false,
			name: '',
			value: '',
			status: 200,
			delay_ms: 100,
			path: '',
			json_value: '{}'
		};
	}

	function removeModification(index: number) {
		rule.modifications.splice(index, 1);
	}

	function getModificationSummary(mod: Modification): string {
		switch (mod.type) {
			case 'replace_body':
				return `Replace "${mod.pattern}" with "${mod.replacement}"${mod.use_regex ? ' (regex)' : ''}`;
			case 'add_header':
				return `Add header: ${mod.name} = ${mod.value}`;
			case 'remove_header':
				return `Remove header: ${mod.name}`;
			case 'change_status':
				return `Change status to ${mod.status}`;
			case 'inject_delay':
				return `Add ${mod.delay_ms}ms delay`;
			case 'modify_json':
				return `Modify JSON at path: ${mod.path}`;
			default:
				return 'Unknown modification';
		}
	}
</script>

<div class="container mx-auto px-4 py-8 max-w-4xl">
	<div class="flex items-center gap-4 mb-6">
		<button onclick={() => goto('/modifiers')} class="text-blue-600 hover:text-blue-700">
			← Back
		</button>
		<h1 class="text-3xl font-bold">Create Response Modifier</h1>
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
				<label class="block text-sm font-medium text-gray-700 mb-2"> Rule Name </label>
				<input
					type="text"
					bind:value={rule.name}
					placeholder="e.g., Mock API Response"
					class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
				/>
			</div>
			<div>
				<label class="block text-sm font-medium text-gray-700 mb-2"> Priority </label>
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
					<label class="block text-sm font-medium text-gray-700 mb-2"> HTTP Method (Optional) </label>
					<select
						bind:value={rule.match_request.method}
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
					<label class="block text-sm font-medium text-gray-700 mb-2"> Match Type </label>
					<select
						bind:value={rule.match_request.url_match_type}
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

			<div class="mb-4">
				<label class="block text-sm font-medium text-gray-700 mb-2"> URL Pattern </label>
				<input
					type="text"
					bind:value={rule.match_request.url_pattern}
					placeholder="e.g., /api/users"
					class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
				/>
			</div>

			<div>
				<label class="block text-sm font-medium text-gray-700 mb-2">
					Status Codes (Optional, comma-separated)
				</label>
				<input
					type="text"
					bind:value={statusCodesText}
					placeholder="e.g., 200, 201, 204"
					class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
				/>
				<p class="text-xs text-gray-500 mt-1">Leave empty to match all status codes</p>
			</div>
		</div>

		<!-- Modifications -->
		<div class="border-t pt-6">
			<h3 class="text-lg font-semibold mb-4">Response Modifications</h3>

			<!-- Current Modifications List -->
			{#if rule.modifications.length > 0}
				<div class="space-y-2 mb-4">
					{#each rule.modifications as mod, index}
						<div class="flex items-center justify-between bg-purple-50 p-3 rounded">
							<div>
								<span class="text-sm font-medium text-purple-700">{mod.type}</span>
								<p class="text-xs text-gray-600">{getModificationSummary(mod)}</p>
							</div>
							<button
								onclick={() => removeModification(index)}
								class="text-red-600 hover:text-red-700"
							>
								Remove
							</button>
						</div>
					{/each}
				</div>
			{/if}

			<!-- Add Modification -->
			<div class="bg-gray-50 p-4 rounded-lg">
				<h4 class="text-sm font-semibold mb-3">Add Modification</h4>

				<div class="mb-3">
					<label class="block text-sm font-medium text-gray-700 mb-2"> Modification Type </label>
					<select
						bind:value={currentModType}
						class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
					>
						<option value="replace_body">Replace Body Content</option>
						<option value="add_header">Add Header</option>
						<option value="remove_header">Remove Header</option>
						<option value="change_status">Change Status Code</option>
						<option value="inject_delay">Inject Delay</option>
						<option value="modify_json">Modify JSON Field</option>
					</select>
				</div>

				<!-- Dynamic form based on modification type -->
				{#if currentModType === 'replace_body'}
					<div class="space-y-3">
						<input
							type="text"
							bind:value={modConfig.pattern}
							placeholder="Pattern to find"
							class="w-full px-4 py-2 border border-gray-300 rounded-lg"
						/>
						<input
							type="text"
							bind:value={modConfig.replacement}
							placeholder="Replacement text"
							class="w-full px-4 py-2 border border-gray-300 rounded-lg"
						/>
						<label class="flex items-center gap-2">
							<input type="checkbox" bind:checked={modConfig.use_regex} class="w-4 h-4" />
							<span class="text-sm">Use Regex</span>
						</label>
					</div>
				{:else if currentModType === 'add_header'}
					<div class="space-y-3">
						<input
							type="text"
							bind:value={modConfig.name}
							placeholder="Header name"
							class="w-full px-4 py-2 border border-gray-300 rounded-lg"
						/>
						<input
							type="text"
							bind:value={modConfig.value}
							placeholder="Header value"
							class="w-full px-4 py-2 border border-gray-300 rounded-lg"
						/>
					</div>
				{:else if currentModType === 'remove_header'}
					<input
						type="text"
						bind:value={modConfig.name}
						placeholder="Header name to remove"
						class="w-full px-4 py-2 border border-gray-300 rounded-lg"
					/>
				{:else if currentModType === 'change_status'}
					<input
						type="number"
						bind:value={modConfig.status}
						placeholder="New status code"
						min="100"
						max="599"
						class="w-full px-4 py-2 border border-gray-300 rounded-lg"
					/>
				{:else if currentModType === 'inject_delay'}
					<input
						type="number"
						bind:value={modConfig.delay_ms}
						placeholder="Delay in milliseconds"
						min="0"
						class="w-full px-4 py-2 border border-gray-300 rounded-lg"
					/>
				{:else if currentModType === 'modify_json'}
					<div class="space-y-3">
						<input
							type="text"
							bind:value={modConfig.path}
							placeholder="JSON path (e.g., user.name)"
							class="w-full px-4 py-2 border border-gray-300 rounded-lg"
						/>
						<textarea
							bind:value={modConfig.json_value}
							placeholder='New value as JSON (e.g., "John Doe" or 123)'
							rows="3"
							class="w-full px-4 py-2 border border-gray-300 rounded-lg font-mono text-sm"
						></textarea>
					</div>
				{/if}

				<button
					onclick={addModification}
					class="mt-3 w-full px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700"
				>
					Add Modification
				</button>
			</div>
		</div>

		<!-- Enable Rule -->
		<div class="border-t pt-6">
			<label class="flex items-center gap-2 cursor-pointer">
				<input type="checkbox" bind:checked={rule.enabled} class="w-4 h-4" />
				<span class="text-sm font-medium text-gray-700">Enable this rule</span>
			</label>
		</div>

		<!-- Actions -->
		<div class="flex gap-4 pt-4 border-t">
			<button
				onclick={save}
				disabled={saving}
				class="flex-1 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
			>
				{saving ? 'Creating...' : 'Create Modifier'}
			</button>
			<button
				onclick={() => goto('/modifiers')}
				class="px-6 py-3 border border-gray-300 rounded-lg hover:bg-gray-50"
			>
				Cancel
			</button>
		</div>
	</div>
</div>
