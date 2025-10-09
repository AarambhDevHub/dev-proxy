<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import type { LatencyRule, LatencyStats } from "$lib/types";

    let rules: LatencyRule[] = $state([]);
    let stats: LatencyStats | null = $state(null);
    let loading = $state(true);

    async function fetchRules() {
        try {
            const res = await fetch("/api/latency-rules");
            rules = await res.json();
        } catch (error) {
            console.error("Failed to fetch latency rules:", error);
        } finally {
            loading = false;
        }
    }

    async function fetchStats() {
        try {
            const res = await fetch("/api/latency-stats");
            stats = await res.json();
        } catch (error) {
            console.error("Failed to fetch stats:", error);
        }
    }

    async function toggleRule(id: string) {
        try {
            await fetch(`/api/latency-rules/${id}/toggle`, { method: "POST" });
            await fetchRules();
        } catch (error) {
            console.error("Failed to toggle rule:", error);
        }
    }

    async function deleteRule(id: string) {
        if (!confirm("Delete this latency rule?")) return;

        try {
            await fetch(`/api/latency-rules/${id}`, { method: "DELETE" });
            await fetchRules();
            await fetchStats();
        } catch (error) {
            console.error("Failed to delete rule:", error);
        }
    }

    async function clearAll() {
        if (!confirm("Delete all latency rules?")) return;

        try {
            await fetch("/api/latency-rules", { method: "DELETE" });
            rules = [];
            stats = null;
            await fetchStats();
        } catch (error) {
            console.error("Failed to clear rules:", error);
        }
    }

    async function resetStats() {
        if (!confirm("Reset latency statistics?")) return;

        try {
            await fetch("/api/latency-stats/reset", { method: "POST" });
            await fetchStats();
        } catch (error) {
            console.error("Failed to reset stats:", error);
        }
    }

    function getDelayDescription(delay: any): string {
        switch (delay.type) {
            case "fixed":
                return `${delay.delay_ms}ms fixed`;
            case "random":
                return `${delay.min_ms}-${delay.max_ms}ms random`;
            case "normal":
                return `${delay.mean_ms}ms ±${delay.std_dev_ms}ms normal`;
            case "spike":
                return `${delay.base_delay_ms}ms with ${delay.spike_delay_ms}ms spikes (${(delay.spike_probability * 100).toFixed(1)}%)`;
            default:
                return "Unknown";
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
            <h1 class="text-3xl font-bold">Latency Injection</h1>
            <p class="text-gray-600 mt-2">
                Simulate network delays and slow responses
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
            {#if stats && stats.total_injections > 0}
                <button
                    onclick={resetStats}
                    class="px-4 py-2 bg-yellow-600 text-white rounded hover:bg-yellow-700"
                >
                    Reset Stats
                </button>
            {/if}
            {#if rules.length > 0}
                <button
                    onclick={clearAll}
                    class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
                >
                    Clear All
                </button>
            {/if}
            <button
                onclick={() => goto("/latency/new")}
                class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700"
            >
                + New Rule
            </button>
        </div>
    </div>

    {#if stats && stats.total_injections > 0}
        <div class="grid grid-cols-5 gap-4 mb-6">
            <div class="bg-white p-6 rounded-lg shadow">
                <h3 class="text-sm font-medium text-gray-600 mb-2">
                    Total Injections
                </h3>
                <p class="text-3xl font-bold text-blue-600">
                    {stats.total_injections.toLocaleString()}
                </p>
            </div>
            <div class="bg-white p-6 rounded-lg shadow">
                <h3 class="text-sm font-medium text-gray-600 mb-2">
                    Avg Delay
                </h3>
                <p class="text-3xl font-bold text-purple-600">
                    {stats.avg_delay_ms}ms
                </p>
            </div>
            <div class="bg-white p-6 rounded-lg shadow">
                <h3 class="text-sm font-medium text-gray-600 mb-2">
                    Min Delay
                </h3>
                <p class="text-3xl font-bold text-green-600">
                    {stats.min_delay_ms}ms
                </p>
            </div>
            <div class="bg-white p-6 rounded-lg shadow">
                <h3 class="text-sm font-medium text-gray-600 mb-2">
                    Max Delay
                </h3>
                <p class="text-3xl font-bold text-red-600">
                    {stats.max_delay_ms}ms
                </p>
            </div>
            <div class="bg-white p-6 rounded-lg shadow">
                <h3 class="text-sm font-medium text-gray-600 mb-2">
                    Total Time Added
                </h3>
                <p class="text-3xl font-bold text-orange-600">
                    {(stats.total_delay_ms / 1000).toFixed(2)}s
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
                No latency rules
            </h3>
            <p class="mt-1 text-gray-500">
                Get started by creating a new latency injection rule.
            </p>
            <button
                onclick={() => goto("/latency/new")}
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
                            >Delay</th
                        >
                        <th
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase"
                            >Apply To</th
                        >
                        <th
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase"
                            >Priority</th
                        >
                        <th
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase"
                            >Stats</th
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
                                    {getDelayDescription(rule.delay)}
                                </div>
                            </td>
                            <td class="px-6 py-4">
                                <span
                                    class="px-2 py-1 text-xs font-semibold rounded-full bg-indigo-100 text-indigo-800 capitalize"
                                >
                                    {rule.match_request.apply_to}
                                </span>
                            </td>
                            <td class="px-6 py-4 text-sm text-gray-900"
                                >{rule.priority}</td
                            >
                            <td class="px-6 py-4">
                                {#if stats && stats.by_rule[rule.id]}
                                    <div class="text-sm text-gray-900">
                                        {stats.by_rule[rule.id].hits} hits
                                    </div>
                                    <div class="text-xs text-gray-500">
                                        {stats.by_rule[rule.id].avg_delay_ms}ms
                                        avg
                                    </div>
                                {:else}
                                    <span class="text-xs text-gray-400"
                                        >No hits yet</span
                                    >
                                {/if}
                            </td>
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
                                    onclick={() => goto(`/latency/${rule.id}`)}
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
