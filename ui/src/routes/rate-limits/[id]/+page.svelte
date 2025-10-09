<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import type { RateLimitRule, KeyType } from "$lib/types";

    let rule: RateLimitRule | null = $state(null);
    let headersText = $state("");
    let keyType = $state("global");
    let headerName = $state("");
    let customPattern = $state("");
    let loading = $state(true);
    let saving = $state(false);
    let error = $state("");

    const id = $derived($page.params.id);

    async function fetchRule() {
        try {
            const res = await fetch(`/api/rate-limits/${id}`);
            if (res.ok) {
                rule = await res.json();
                if (rule) {
                    headersText = Object.entries(rule.response.headers)
                        .map(([k, v]) => `${k}: ${v}`)
                        .join("\n");

                    // Extract key type info
                    const kt = rule.match_request.key_type;
                    if (kt === "global") {
                        keyType = "global";
                    } else if (kt === "ipaddress") {
                        keyType = "ipaddress";
                    } else if (typeof kt === "object" && "header" in kt) {
                        keyType = "header";
                        headerName = kt.header.name;
                    } else if (typeof kt === "object" && "custom" in kt) {
                        keyType = "custom";
                        customPattern = kt.custom.pattern;
                    }
                }
            } else {
                error = "Rate limit rule not found";
            }
        } catch (e) {
            error = "Failed to load rate limit rule";
        } finally {
            loading = false;
        }
    }

    function updateKeyType() {
        if (!rule) return;

        if (keyType === "global") {
            rule.match_request.key_type = "global" as KeyType;
        } else if (keyType === "ipaddress") {
            rule.match_request.key_type = "ipaddress" as KeyType;
        } else if (keyType === "header") {
            rule.match_request.key_type = { header: { name: headerName } };
        } else if (keyType === "custom") {
            rule.match_request.key_type = {
                custom: { pattern: customPattern },
            };
        }
    }

    async function save() {
        if (!rule || !rule.name || !rule.match_request.url_pattern) {
            error = "Name and URL pattern are required";
            return;
        }

        if (rule.limit.max_requests <= 0 || rule.limit.window_seconds <= 0) {
            error = "Limit values must be positive";
            return;
        }

        saving = true;
        error = "";

        try {
            updateKeyType();

            const headers: Record<string, string> = {};
            headersText.split("\n").forEach((line) => {
                const [key, ...valueParts] = line.split(":");
                if (key && valueParts.length > 0) {
                    headers[key.trim()] = valueParts.join(":").trim();
                }
            });

            const payload = {
                id: rule.id,
                name: rule.name,
                enabled: rule.enabled,
                priority: rule.priority,
                match_request: rule.match_request,
                limit: rule.limit,
                response: {
                    ...rule.response,
                    headers,
                },
            };

            const res = await fetch(`/api/rate-limits/${id}`, {
                method: "PUT",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload),
            });

            if (res.ok) {
                goto("/rate-limits");
            } else {
                error = "Failed to update rate limit rule";
            }
        } catch (e) {
            error = "Failed to update rate limit rule: " + e;
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
            onclick={() => goto("/rate-limits")}
            class="text-blue-600 hover:text-blue-700"
        >
            ← Back
        </button>
        <h1 class="text-3xl font-bold">Edit Rate Limit Rule</h1>
    </div>

    {#if loading}
        <div class="text-center py-12">
            <div
                class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"
            ></div>
            <p class="mt-4 text-gray-600">Loading...</p>
        </div>
    {:else if error && !rule}
        <div class="text-center py-12 bg-white rounded-lg shadow">
            <p class="text-red-600">{error}</p>
        </div>
    {:else if rule}
        {#if error}
            <div
                class="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700"
            >
                {error}
            </div>
        {/if}

        <!-- Same form as new page, but with rule data bound -->
        <div class="bg-white rounded-lg shadow p-6 space-y-6">
            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                        Rule Name *
                    </label>
                    <input
                        type="text"
                        bind:value={rule.name}
                        placeholder="API Rate Limit"
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

            <div>
                <label class="flex items-center gap-2">
                    <input
                        type="checkbox"
                        bind:checked={rule.enabled}
                        class="w-4 h-4"
                    />
                    <span class="text-sm font-medium text-gray-700"
                        >Enabled</span
                    >
                </label>
            </div>

            <hr />

            <div class="space-y-4">
                <h3 class="text-lg font-semibold">Request Matching</h3>

                <div class="grid grid-cols-3 gap-4">
                    <div>
                        <label
                            class="block text-sm font-medium text-gray-700 mb-2"
                        >
                            HTTP Method (Optional)
                        </label>
                        <select
                            bind:value={rule.match_request.method}
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                        >
                            <option value={undefined}>Any</option>
                            <option value="GET">GET</option>
                            <option value="POST">POST</option>
                            <option value="PUT">PUT</option>
                            <option value="DELETE">DELETE</option>
                            <option value="PATCH">PATCH</option>
                        </select>
                    </div>

                    <div>
                        <label
                            class="block text-sm font-medium text-gray-700 mb-2"
                        >
                            URL Pattern *
                        </label>
                        <input
                            type="text"
                            bind:value={rule.match_request.url_pattern}
                            placeholder="/api/"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>

                    <div>
                        <label
                            class="block text-sm font-medium text-gray-700 mb-2"
                        >
                            Match Type
                        </label>
                        <select
                            bind:value={rule.match_request.url_match_type}
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                        >
                            <option value="exact">Exact</option>
                            <option value="contains">Contains</option>
                            <option value="startswith">Starts With</option>
                            <option value="endswith">Ends With</option>
                            <option value="regex">Regex</option>
                        </select>
                    </div>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                        Rate Limit Key
                    </label>
                    <select
                        bind:value={keyType}
                        onchange={updateKeyType}
                        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                    >
                        <option value="global"
                            >Global (shared across all clients)</option
                        >
                        <option value="ipaddress">Per IP Address</option>
                        <option value="header">Per Header Value</option>
                        <option value="custom">Custom Pattern</option>
                    </select>

                    {#if keyType === "header"}
                        <input
                            type="text"
                            bind:value={headerName}
                            onchange={updateKeyType}
                            placeholder="Header name (e.g., X-API-Key)"
                            class="mt-2 w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    {:else if keyType === "custom"}
                        <input
                            type="text"
                            bind:value={customPattern}
                            onchange={updateKeyType}
                            placeholder="Custom pattern"
                            class="mt-2 w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    {/if}
                </div>
            </div>

            <hr />

            <div class="space-y-4">
                <h3 class="text-lg font-semibold">Rate Limit Configuration</h3>

                <div class="grid grid-cols-3 gap-4">
                    <div>
                        <label
                            class="block text-sm font-medium text-gray-700 mb-2"
                        >
                            Max Requests *
                        </label>
                        <input
                            type="number"
                            bind:value={rule.limit.max_requests}
                            min="1"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>

                    <div>
                        <label
                            class="block text-sm font-medium text-gray-700 mb-2"
                        >
                            Window (seconds) *
                        </label>
                        <input
                            type="number"
                            bind:value={rule.limit.window_seconds}
                            min="1"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>

                    <div>
                        <label
                            class="block text-sm font-medium text-gray-700 mb-2"
                        >
                            Burst Size (optional)
                        </label>
                        <input
                            type="number"
                            bind:value={rule.limit.burst_size}
                            min="0"
                            placeholder="0"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>
                </div>

                <div class="p-4 bg-blue-50 border border-blue-200 rounded-lg">
                    <p class="text-sm text-blue-800">
                        This rule will allow <strong
                            >{rule.limit.max_requests} requests per {rule.limit
                                .window_seconds} seconds</strong
                        >.
                        {#if rule.limit.burst_size}
                            <br />With burst capacity of
                            <strong
                                >{rule.limit.burst_size} extra requests</strong
                            >.
                        {/if}
                    </p>
                </div>
            </div>

            <hr />

            <div class="space-y-4">
                <h3 class="text-lg font-semibold">Rate Limit Response</h3>

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label
                            class="block text-sm font-medium text-gray-700 mb-2"
                        >
                            Status Code
                        </label>
                        <input
                            type="number"
                            bind:value={rule.response.status}
                            min="400"
                            max="599"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>

                    <div>
                        <label
                            class="block text-sm font-medium text-gray-700 mb-2"
                        >
                            Delay (ms, optional)
                        </label>
                        <input
                            type="number"
                            bind:value={rule.response.delay_ms}
                            min="0"
                            placeholder="0"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                        Response Headers (one per line: Name: Value)
                    </label>
                    <textarea
                        bind:value={headersText}
                        rows="3"
                        class="w-full px-4 py-2 border border-gray-300 rounded-lg font-mono text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
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
                        rows="8"
                        class="w-full px-4 py-2 border border-gray-300 rounded-lg font-mono text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                    ></textarea>
                </div>
            </div>

            <div class="flex justify-end gap-4">
                <button
                    onclick={() => goto("/rate-limits")}
                    class="px-6 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
                >
                    Cancel
                </button>
                <button
                    onclick={save}
                    disabled={saving}
                    class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
                >
                    {saving ? "Saving..." : "Save Changes"}
                </button>
            </div>
        </div>
    {/if}
</div>
