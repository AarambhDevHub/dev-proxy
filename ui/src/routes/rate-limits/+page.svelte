<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import type { RateLimitRule, BucketStats } from "$lib/types";

    let rules: RateLimitRule[] = $state([]);
    let stats: BucketStats | null = $state(null);
    let loading = $state(true);

    async function fetchRules() {
        try {
            const res = await fetch("/api/rate-limits");
            rules = await res.json();
        } catch (error) {
            console.error("Failed to fetch rate limit rules:", error);
        } finally {
            loading = false;
        }
    }

    async function fetchStats() {
        try {
            const res = await fetch("/api/rate-limits/stats");
            stats = await res.json();
        } catch (error) {
            console.error("Failed to fetch stats:", error);
        }
    }

    async function toggleRule(id: string) {
        try {
            await fetch(`/api/rate-limits/${id}/toggle`, { method: "POST" });
            await fetchRules();
        } catch (error) {
            console.error("Failed to toggle rule:", error);
        }
    }

    async function resetBucket(id: string) {
        if (!confirm("Reset rate limit counters for this rule?")) return;

        try {
            await fetch(`/api/rate-limits/${id}/reset`, { method: "POST" });
            alert("Rate limit counters reset successfully");
            await fetchStats();
        } catch (error) {
            console.error("Failed to reset bucket:", error);
        }
    }

    async function deleteRule(id: string) {
        if (!confirm("Delete this rate limit rule?")) return;

        try {
            await fetch(`/api/rate-limits/${id}`, { method: "DELETE" });
            await fetchRules();
            await fetchStats();
        } catch (error) {
            console.error("Failed to delete rule:", error);
        }
    }

    async function clearAll() {
        if (!confirm("Delete all rate limit rules?")) return;

        try {
            await fetch("/api/rate-limits", { method: "DELETE" });
            rules = [];
            stats = null;
            await fetchStats();
        } catch (error) {
            console.error("Failed to clear rules:", error);
        }
    }

    onMount(() => {
        fetchRules();
        fetchStats();
    });
</script>

<div class="container mx-auto px-4 py-8">
    <div class="flex justify-between items-center mb-6">
        <div>
            <h1 class="text-3xl font-bold">Rate Limit Simulation</h1>
            <p class="text-gray-600 mt-2">
                Test rate limiting behavior for your applications
            </p>
        </div>
        <div class="flex gap-4">
            <button
                onclick={() => {
                    fetchRules();
                    fetchStats();
                }}
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
                onclick={() => goto("/rate-limits/new")}
                class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700"
            >
                + New Rule
            </button>
        </div>
    </div>

    {#if stats}
        <div class="grid grid-cols-2 gap-4 mb-6">
            <div class="bg-white p-6 rounded-lg shadow">
                <h3 class="text-sm font-medium text-gray-600 mb-2">
                    Active Buckets
                </h3>
                <p class="text-3xl font-bold text-blue-600">
                    {stats.active_limits}
                </p>
            </div>
            <div class="bg-white p-6 rounded-lg shadow">
                <h3 class="text-sm font-medium text-gray-600 mb-2">
                    Total Trackers
                </h3>
                <p class="text-3xl font-bold text-purple-600">
                    {stats.total_buckets}
                </p>
            </div>
        </div>
    {/if}

    {#if loading}
        <div class="text-center py-12">
            <div
                class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"
            ></div>
            <p class="mt-4 text-gray-600">Loading rules...</p>
        </div>
    {:else if rules.length === 0}
        <div class="text-center py-12 bg-white rounded-lg shadow">
            <svg
                class="mx-auto h-12 w-12 text-gray-400"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
                />
            </svg>
            <h3 class="mt-2 text-lg font-medium text-gray-900">
                No rate limit rules
            </h3>
            <p class="mt-1 text-gray-500">
                Get started by creating a new rate limit rule.
            </p>
            <button
                onclick={() => goto("/rate-limits/new")}
                class="mt-6 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
            >
                Create Rule
            </button>
        </div>
    {:else}
        <div class="bg-white rounded-lg shadow overflow-hidden">
            <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                    <tr>
                        <th
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase"
                            >Name</th
                        >
                        <th
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase"
                            >Pattern</th
                        >
                        <th
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase"
                            >Limit</th
                        >
                        <th
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase"
                            >Key Type</th
                        >
                        <th
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase"
                            >Priority</th
                        >
                        <th
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase"
                            >Status</th
                        >
                        <th
                            class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase"
                            >Actions</th
                        >
                    </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                    {#each rules as rule}
                        <tr class="hover:bg-gray-50">
                            <td class="px-6 py-4">
                                <div class="text-sm font-medium text-gray-900">
                                    {rule.name}
                                </div>
                                {#if rule.match_request.method}
                                    <span class="text-xs text-gray-500"
                                        >{rule.match_request.method}</span
                                    >
                                {/if}
                            </td>
                            <td class="px-6 py-4">
                                <div class="text-sm text-gray-900 font-mono">
                                    {rule.match_request.url_pattern}
                                </div>
                                <span class="text-xs text-gray-500"
                                    >{rule.match_request.url_match_type}</span
                                >
                            </td>
                            <td class="px-6 py-4">
                                <div class="text-sm text-gray-900">
                                    {rule.limit.max_requests} req / {rule.limit
                                        .window_seconds}s
                                </div>
                                {#if rule.limit.burst_size}
                                    <span class="text-xs text-blue-600"
                                        >Burst: {rule.limit.burst_size}</span
                                    >
                                {/if}
                            </td>
                            <td class="px-6 py-4">
                                <span class="text-sm text-gray-600 capitalize">
                                    {#if rule.match_request.key_type === "global"}
                                        Global
                                    {:else if rule.match_request.key_type === "ipaddress"}
                                        IP Address
                                    {:else if typeof rule.match_request.key_type === "object" && "header" in rule.match_request.key_type}
                                        Header: {rule.match_request.key_type
                                            .header.name}
                                    {:else if typeof rule.match_request.key_type === "object" && "custom" in rule.match_request.key_type}
                                        Custom: {rule.match_request.key_type
                                            .custom.pattern}
                                    {/if}
                                </span>
                            </td>
                            <td class="px-6 py-4 text-sm text-gray-900"
                                >{rule.priority}</td
                            >
                            <td class="px-6 py-4">
                                <span
                                    class="px-2 py-1 text-xs font-semibold rounded-full {rule.enabled
                                        ? 'bg-green-100 text-green-800'
                                        : 'bg-gray-100 text-gray-800'}"
                                >
                                    {rule.enabled ? "Enabled" : "Disabled"}
                                </span>
                            </td>
                            <td
                                class="px-6 py-4 text-right text-sm font-medium space-x-2"
                            >
                                <button
                                    onclick={() => toggleRule(rule.id)}
                                    class="text-blue-600 hover:text-blue-900"
                                >
                                    {rule.enabled ? "Disable" : "Enable"}
                                </button>
                                <button
                                    onclick={() => resetBucket(rule.id)}
                                    class="text-yellow-600 hover:text-yellow-900"
                                >
                                    Reset
                                </button>
                                <button
                                    onclick={() =>
                                        goto(`/rate-limits/${rule.id}`)}
                                    class="text-indigo-600 hover:text-indigo-900"
                                >
                                    Edit
                                </button>
                                <button
                                    onclick={() => deleteRule(rule.id)}
                                    class="text-red-600 hover:text-red-900"
                                >
                                    Delete
                                </button>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>
