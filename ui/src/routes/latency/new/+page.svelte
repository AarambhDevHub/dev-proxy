<script lang="ts">
    import { goto } from "$app/navigation";
    import type { CreateLatencyRule, DelayConfig } from "$lib/types";

    let rule: CreateLatencyRule = $state({
        name: "",
        enabled: true,
        priority: 100,
        match_request: {
            method: undefined,
            url_pattern: "/api/",
            url_match_type: "startswith",
            apply_to: "both",
        },
        delay: {
            type: "fixed",
            delay_ms: 1000,
        },
    });

    let delayType = $state("fixed");
    let saving = $state(false);
    let error = $state("");

    // Delay config state
    let fixedDelay = $state(1000);
    let randomMin = $state(500);
    let randomMax = $state(2000);
    let normalMean = $state(1000);
    let normalStdDev = $state(200);
    let spikeBase = $state(100);
    let spikeDelay = $state(5000);
    let spikeProbability = $state(0.1);

    function updateDelayConfig() {
        if (delayType === "fixed") {
            rule.delay = {
                type: "fixed",
                delay_ms: fixedDelay,
            };
        } else if (delayType === "random") {
            rule.delay = {
                type: "random",
                min_ms: randomMin,
                max_ms: randomMax,
            };
        } else if (delayType === "normal") {
            rule.delay = {
                type: "normal",
                mean_ms: normalMean,
                std_dev_ms: normalStdDev,
            };
        } else if (delayType === "spike") {
            rule.delay = {
                type: "spike",
                base_delay_ms: spikeBase,
                spike_delay_ms: spikeDelay,
                spike_probability: spikeProbability,
            };
        }
    }

    async function save() {
        if (!rule.name || !rule.match_request.url_pattern) {
            error = "Name and URL pattern are required";
            return;
        }

        // Validate delay values
        if (delayType === "fixed" && fixedDelay < 0) {
            error = "Delay must be non-negative";
            return;
        }

        if (delayType === "random") {
            if (randomMin < 0 || randomMax < 0) {
                error = "Delay values must be non-negative";
                return;
            }
            if (randomMin > randomMax) {
                error = "Min delay must be less than or equal to max delay";
                return;
            }
        }

        if (delayType === "normal") {
            if (normalMean < 0 || normalStdDev < 0) {
                error = "Mean and standard deviation must be non-negative";
                return;
            }
        }

        if (delayType === "spike") {
            if (spikeBase < 0 || spikeDelay < 0) {
                error = "Delay values must be non-negative";
                return;
            }
            if (spikeProbability < 0 || spikeProbability > 1) {
                error = "Spike probability must be between 0 and 1";
                return;
            }
        }

        saving = true;
        error = "";

        try {
            updateDelayConfig();

            console.log("Sending payload:", JSON.stringify(rule, null, 2));

            const res = await fetch("/api/latency-rules", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(rule),
            });

            if (res.ok) {
                goto("/latency");
            } else {
                const errorData = await res.json();
                error = errorData.error || "Failed to create latency rule";
            }
        } catch (e) {
            error = "Failed to create latency rule: " + e;
        } finally {
            saving = false;
        }
    }
</script>

<div class="container mx-auto px-4 py-8 max-w-4xl">
    <div class="flex items-center gap-4 mb-6">
        <button
            onclick={() => goto("/latency")}
            class="text-blue-600 hover:text-blue-700"
        >
            ← Back
        </button>
        <h1 class="text-3xl font-bold">New Latency Injection Rule</h1>
    </div>

    {#if error}
        <div
            class="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700"
        >
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
                    placeholder="Slow API Response"
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

        <!-- Request Matching -->
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
                    Apply Delay To
                </label>
                <select
                    bind:value={rule.match_request.apply_to}
                    class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                    <option value="request"
                        >Request (before proxying to upstream)</option
                    >
                    <option value="response"
                        >Response (before sending to client)</option
                    >
                    <option value="both">Both (request and response)</option>
                </select>
            </div>
        </div>

        <hr />

        <!-- Delay Configuration -->
        <div class="space-y-4">
            <h3 class="text-lg font-semibold">Delay Configuration</h3>

            <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">
                    Delay Type
                </label>
                <select
                    bind:value={delayType}
                    onchange={updateDelayConfig}
                    class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                    <option value="fixed">Fixed Delay</option>
                    <option value="random">Random Delay</option>
                    <option value="normal">Normal Distribution</option>
                    <option value="spike">Spike Pattern</option>
                </select>
            </div>

            <!-- Fixed Delay -->
            {#if delayType === "fixed"}
                <div
                    class="p-4 bg-blue-50 border border-blue-200 rounded-lg space-y-4"
                >
                    <div>
                        <label
                            class="block text-sm font-medium text-gray-700 mb-2"
                        >
                            Delay (milliseconds)
                        </label>
                        <input
                            type="number"
                            bind:value={fixedDelay}
                            onchange={updateDelayConfig}
                            min="0"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>
                    <p class="text-sm text-blue-800">
                        All matching requests will be delayed by exactly <strong
                            >{fixedDelay}ms</strong
                        >.
                    </p>
                </div>
            {/if}

            <!-- Random Delay -->
            {#if delayType === "random"}
                <div
                    class="p-4 bg-purple-50 border border-purple-200 rounded-lg space-y-4"
                >
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label
                                class="block text-sm font-medium text-gray-700 mb-2"
                            >
                                Min Delay (ms)
                            </label>
                            <input
                                type="number"
                                bind:value={randomMin}
                                onchange={updateDelayConfig}
                                min="0"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
                            />
                        </div>
                        <div>
                            <label
                                class="block text-sm font-medium text-gray-700 mb-2"
                            >
                                Max Delay (ms)
                            </label>
                            <input
                                type="number"
                                bind:value={randomMax}
                                onchange={updateDelayConfig}
                                min="0"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
                            />
                        </div>
                    </div>
                    <p class="text-sm text-purple-800">
                        Each request will be delayed by a random value between <strong
                            >{randomMin}ms</strong
                        >
                        and <strong>{randomMax}ms</strong>.
                    </p>
                </div>
            {/if}

            <!-- Normal Distribution -->
            {#if delayType === "normal"}
                <div
                    class="p-4 bg-green-50 border border-green-200 rounded-lg space-y-4"
                >
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label
                                class="block text-sm font-medium text-gray-700 mb-2"
                            >
                                Mean Delay (ms)
                            </label>
                            <input
                                type="number"
                                bind:value={normalMean}
                                onchange={updateDelayConfig}
                                min="0"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-green-500"
                            />
                        </div>
                        <div>
                            <label
                                class="block text-sm font-medium text-gray-700 mb-2"
                            >
                                Std Deviation (ms)
                            </label>
                            <input
                                type="number"
                                bind:value={normalStdDev}
                                onchange={updateDelayConfig}
                                min="0"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-green-500"
                            />
                        </div>
                    </div>
                    <p class="text-sm text-green-800">
                        Delays will follow a normal distribution with mean <strong
                            >{normalMean}ms</strong
                        >
                        and standard deviation
                        <strong>{normalStdDev}ms</strong>. Most delays will be
                        between {Math.max(0, normalMean - normalStdDev)}ms and {normalMean +
                            normalStdDev}ms.
                    </p>
                </div>
            {/if}

            <!-- Spike Pattern -->
            {#if delayType === "spike"}
                <div
                    class="p-4 bg-orange-50 border border-orange-200 rounded-lg space-y-4"
                >
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label
                                class="block text-sm font-medium text-gray-700 mb-2"
                            >
                                Base Delay (ms)
                            </label>
                            <input
                                type="number"
                                bind:value={spikeBase}
                                onchange={updateDelayConfig}
                                min="0"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-orange-500"
                            />
                        </div>
                        <div>
                            <label
                                class="block text-sm font-medium text-gray-700 mb-2"
                            >
                                Spike Delay (ms)
                            </label>
                            <input
                                type="number"
                                bind:value={spikeDelay}
                                onchange={updateDelayConfig}
                                min="0"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-orange-500"
                            />
                        </div>
                    </div>
                    <div>
                        <label
                            class="block text-sm font-medium text-gray-700 mb-2"
                        >
                            Spike Probability (0.0 - 1.0)
                        </label>
                        <input
                            type="number"
                            bind:value={spikeProbability}
                            onchange={updateDelayConfig}
                            min="0"
                            max="1"
                            step="0.01"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-orange-500"
                        />
                    </div>
                    <p class="text-sm text-orange-800">
                        Requests will normally have <strong
                            >{spikeBase}ms</strong
                        >
                        delay, but
                        <strong>{(spikeProbability * 100).toFixed(1)}%</strong>
                        of requests will have a spike delay of
                        <strong>{spikeDelay}ms</strong>.
                    </p>
                </div>
            {/if}
        </div>

        <div class="flex justify-end gap-4">
            <button
                onclick={() => goto("/latency")}
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
