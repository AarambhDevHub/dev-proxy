<script lang="ts">
    import { goto } from "$app/navigation";
    import type { CreateRateLimitRule, KeyType } from "$lib/types";

    let rule: CreateRateLimitRule = $state({
        name: "",
        enabled: true,
        priority: 100,
        match_request: {
            method: undefined,
            url_pattern: "/api/",
            url_match_type: "startswith",
            key_type: "global",
        },
        limit: {
            max_requests: 10,
            window_seconds: 60,
            burst_size: undefined,
        },
        response: {
            status: 429,
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(
                {
                    error: "Rate limit exceeded",
                    message: "Too many requests, please try again later",
                },
                null,
                2,
            ),
            delay_ms: undefined,
        },
    });

    let headersText = $state("Content-Type: application/json");
    let keyType = $state("global");
    let headerName = $state("");
    let customPattern = $state("");
    let saving = $state(false);
    let error = $state("");

    function updateKeyType() {
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
        if (!rule.name || !rule.match_request.url_pattern) {
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
                ...rule,
                response: {
                    ...rule.response,
                    headers,
                },
            };

            const res = await fetch("/api/rate-limits", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload),
            });

            if (res.ok) {
                goto("/rate-limits");
            } else {
                error = "Failed to create rate limit rule";
            }
        } catch (e) {
            error = "Failed to create rate limit rule: " + e;
        } finally {
            saving = false;
        }
    }

    function formatJSON() {
        try {
            const parsed = JSON.parse(rule.response.body);
            rule.response.body = JSON.stringify(parsed, null, 2);
        } catch {
            // Invalid JSON, keep as is
        }
    }
</script>

<div class="container mx-auto px-4 py-8 max-w-4xl">
    <div class="flex items-center gap-4 mb-6">
        <button
            onclick={() => goto("/rate-limits")}
            class="text-blue-600 hover:text-blue-700"
        >
            ← Back
        </button>
        <h1 class="text-3xl font-bold">New Rate Limit Rule</h1>
    </div>

    {#if error}
        <div
            class="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700"
        >
            {error}
        </div>
    {/if}

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
                <span class="text-sm font-medium text-gray-700">Enabled</span>
            </label>
        </div>

        <hr />

        <div class="space-y-4">
            <h3 class="text-lg font-semibold">Request Matching</h3>

            <div class="grid grid-cols-3 gap-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
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
                    <label class="block text-sm font-medium text-gray-700 mb-2">
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
                    <label class="block text-sm font-medium text-gray-700 mb-2">
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
                    <label class="block text-sm font-medium text-gray-700 mb-2">
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
                    <label class="block text-sm font-medium text-gray-700 mb-2">
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
                    <label class="block text-sm font-medium text-gray-700 mb-2">
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
                        <strong>{rule.limit.burst_size} extra requests</strong>.
                    {/if}
                </p>
            </div>
        </div>

        <hr />

        <div class="space-y-4">
            <h3 class="text-lg font-semibold">Rate Limit Response</h3>

            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
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
                    <label class="block text-sm font-medium text-gray-700 mb-2">
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
                {saving ? "Creating..." : "Create Rule"}
            </button>
        </div>
    </div>
</div>
