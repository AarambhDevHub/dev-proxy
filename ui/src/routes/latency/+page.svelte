<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { toast } from "svelte-sonner";
    import * as Card from "$lib/components/ui/card";
    import { Button } from "$lib/components/ui/button";
    import { Badge } from "$lib/components/ui/badge";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Switch } from "$lib/components/ui/switch";
    import * as Dialog from "$lib/components/ui/dialog";
    import * as Table from "$lib/components/ui/table";
    import {
        RefreshCw,
        Trash2,
        Plus,
        Pencil,
        Timer,
        RotateCcw,
        Clock,
        TrendingUp,
    } from "lucide-svelte";
    import type { LatencyRule, LatencyStats } from "$lib/types";

    let rules: LatencyRule[] = $state([]);
    let stats: LatencyStats | null = $state(null);
    let loading = $state(true);
    let deleteDialogOpen = $state(false);
    let ruleToDelete: LatencyRule | null = $state(null);

    async function fetchRules() {
        try {
            const res = await fetch("/api/latency-rules");
            rules = await res.json();
        } catch (error) {
            console.error("Failed to fetch latency rules:", error);
            toast.error("Failed to fetch latency rules");
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
            toast.success("Latency rule toggled");
        } catch (error) {
            console.error("Failed to toggle rule:", error);
            toast.error("Failed to toggle rule");
        }
    }

    async function deleteRule() {
        if (!ruleToDelete) return;
        try {
            await fetch(`/api/latency-rules/${ruleToDelete.id}`, {
                method: "DELETE",
            });
            await fetchRules();
            await fetchStats();
            toast.success("Latency rule deleted");
        } catch (error) {
            console.error("Failed to delete rule:", error);
            toast.error("Failed to delete rule");
        } finally {
            deleteDialogOpen = false;
            ruleToDelete = null;
        }
    }

    async function clearAll() {
        try {
            await fetch("/api/latency-rules", { method: "DELETE" });
            rules = [];
            stats = null;
            await fetchStats();
            toast.success("All latency rules cleared");
        } catch (error) {
            console.error("Failed to clear rules:", error);
            toast.error("Failed to clear rules");
        }
    }

    async function resetStats() {
        try {
            await fetch("/api/latency-stats/reset", { method: "POST" });
            await fetchStats();
            toast.success("Latency statistics reset");
        } catch (error) {
            console.error("Failed to reset stats:", error);
            toast.error("Failed to reset stats");
        }
    }

    function getDelayDescription(delay: any): string {
        switch (delay.type) {
            case "fixed":
                return `${delay.delay_ms}ms fixed`;
            case "random":
                return `${delay.min_ms}-${delay.max_ms}ms random`;
            case "normal":
                return `${delay.mean_ms}ms ±${delay.std_dev_ms}ms`;
            case "spike":
                return `${delay.base_delay_ms}ms with ${delay.spike_delay_ms}ms spikes (${(delay.spike_probability * 100).toFixed(0)}%)`;
            default:
                return "Unknown";
        }
    }

    function confirmDelete(rule: LatencyRule) {
        ruleToDelete = rule;
        deleteDialogOpen = true;
    }

    onMount(() => {
        fetchRules();
        fetchStats();
    });
</script>

<!-- Header -->
<div class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
    <div>
        <h1 class="text-3xl font-bold tracking-tight">Latency Injection</h1>
        <p class="text-muted-foreground">
            Simulate network delays and slow responses
        </p>
    </div>
    <div class="flex items-center gap-2">
        <Button
            variant="outline"
            size="sm"
            onclick={() => {
                fetchRules();
                fetchStats();
            }}
        >
            <RefreshCw class="mr-2 h-4 w-4" />
            Refresh
        </Button>
        {#if stats && stats.total_injections > 0}
            <Button variant="secondary" size="sm" onclick={resetStats}>
                <RotateCcw class="mr-2 h-4 w-4" />
                Reset Stats
            </Button>
        {/if}
        {#if rules.length > 0}
            <Button variant="destructive" size="sm" onclick={clearAll}>
                <Trash2 class="mr-2 h-4 w-4" />
                Clear All
            </Button>
        {/if}
        <Button size="sm" onclick={() => goto("/latency/new")}>
            <Plus class="mr-2 h-4 w-4" />
            New Rule
        </Button>
    </div>
</div>

<!-- Stats -->
{#if stats && stats.total_injections > 0}
    <div class="grid gap-4 md:grid-cols-5 mt-6">
        <Card.Root>
            <Card.Header
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <Card.Title class="text-sm font-medium"
                    >Total Injections</Card.Title
                >
                <Timer class="h-4 w-4 text-muted-foreground" />
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold text-blue-600">
                    {stats.total_injections.toLocaleString()}
                </div>
            </Card.Content>
        </Card.Root>

        <Card.Root>
            <Card.Header
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <Card.Title class="text-sm font-medium">Avg Delay</Card.Title>
                <TrendingUp class="h-4 w-4 text-muted-foreground" />
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold text-purple-600">
                    {stats.avg_delay_ms}ms
                </div>
            </Card.Content>
        </Card.Root>

        <Card.Root>
            <Card.Header
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <Card.Title class="text-sm font-medium">Min Delay</Card.Title>
                <Clock class="h-4 w-4 text-muted-foreground" />
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold text-green-600">
                    {stats.min_delay_ms}ms
                </div>
            </Card.Content>
        </Card.Root>

        <Card.Root>
            <Card.Header
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <Card.Title class="text-sm font-medium">Max Delay</Card.Title>
                <Clock class="h-4 w-4 text-muted-foreground" />
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold text-red-600">
                    {stats.max_delay_ms}ms
                </div>
            </Card.Content>
        </Card.Root>

        <Card.Root>
            <Card.Header
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <Card.Title class="text-sm font-medium">Total Time</Card.Title>
                <Timer class="h-4 w-4 text-muted-foreground" />
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold text-orange-600">
                    {(stats.total_delay_ms / 1000).toFixed(2)}s
                </div>
            </Card.Content>
        </Card.Root>
    </div>
{/if}

{#if loading}
    <Card.Root class="mt-6">
        <Card.Content class="p-6 space-y-4">
            {#each Array(3) as _}
                <div class="flex items-center gap-4">
                    <Skeleton class="h-6 w-32" />
                    <Skeleton class="h-6 w-48" />
                    <Skeleton class="h-6 w-24" />
                    <Skeleton class="h-6 flex-1" />
                </div>
            {/each}
        </Card.Content>
    </Card.Root>
{:else if rules.length === 0}
    <Card.Root class="mt-6">
        <Card.Content class="flex flex-col items-center justify-center py-12">
            <Timer class="h-12 w-12 text-muted-foreground mb-4" />
            <h3 class="text-lg font-semibold">No Latency Rules</h3>
            <p class="text-muted-foreground mb-4">
                Get started by creating a new latency injection rule
            </p>
            <Button onclick={() => goto("/latency/new")}>
                <Plus class="mr-2 h-4 w-4" />
                Create Rule
            </Button>
        </Card.Content>
    </Card.Root>
{:else}
    <Card.Root class="mt-6">
        <Table.Root>
            <Table.Header>
                <Table.Row>
                    <Table.Head>Name</Table.Head>
                    <Table.Head>Pattern</Table.Head>
                    <Table.Head>Delay</Table.Head>
                    <Table.Head>Apply To</Table.Head>
                    <Table.Head>Priority</Table.Head>
                    <Table.Head>Stats</Table.Head>
                    <Table.Head>Status</Table.Head>
                    <Table.Head class="text-right">Actions</Table.Head>
                </Table.Row>
            </Table.Header>
            <Table.Body>
                {#each rules as rule (rule.id)}
                    <Table.Row>
                        <Table.Cell>
                            <div class="font-medium">{rule.name}</div>
                            {#if rule.match_request.method}
                                <span class="text-xs text-muted-foreground"
                                    >{rule.match_request.method}</span
                                >
                            {/if}
                        </Table.Cell>
                        <Table.Cell>
                            <code class="text-sm font-mono"
                                >{rule.match_request.url_pattern}</code
                            >
                            <div class="text-xs text-muted-foreground">
                                {rule.match_request.url_match_type}
                            </div>
                        </Table.Cell>
                        <Table.Cell>
                            <div class="text-sm">
                                {getDelayDescription(rule.delay)}
                            </div>
                        </Table.Cell>
                        <Table.Cell>
                            <Badge variant="secondary" class="capitalize"
                                >{rule.match_request.apply_to}</Badge
                            >
                        </Table.Cell>
                        <Table.Cell>{rule.priority}</Table.Cell>
                        <Table.Cell>
                            {#if stats && stats.by_rule[rule.id]}
                                <div class="text-sm">
                                    {stats.by_rule[rule.id].hits} hits
                                </div>
                                <div class="text-xs text-muted-foreground">
                                    {stats.by_rule[rule.id].avg_delay_ms}ms avg
                                </div>
                            {:else}
                                <span class="text-xs text-muted-foreground"
                                    >No hits</span
                                >
                            {/if}
                        </Table.Cell>
                        <Table.Cell>
                            <Badge
                                variant={rule.enabled ? "default" : "secondary"}
                            >
                                {rule.enabled ? "Enabled" : "Disabled"}
                            </Badge>
                        </Table.Cell>
                        <Table.Cell class="text-right">
                            <div class="flex items-center justify-end gap-1">
                                <Switch
                                    checked={rule.enabled}
                                    onCheckedChange={() => toggleRule(rule.id)}
                                />
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    onclick={() => goto(`/latency/${rule.id}`)}
                                >
                                    <Pencil class="h-4 w-4" />
                                </Button>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    onclick={() => confirmDelete(rule)}
                                >
                                    <Trash2 class="h-4 w-4 text-red-500" />
                                </Button>
                            </div>
                        </Table.Cell>
                    </Table.Row>
                {/each}
            </Table.Body>
        </Table.Root>
    </Card.Root>
{/if}

<!-- Delete Confirmation Dialog -->
<Dialog.Root bind:open={deleteDialogOpen}>
    <Dialog.Content>
        <Dialog.Header>
            <Dialog.Title>Delete Latency Rule</Dialog.Title>
            <Dialog.Description>
                Are you sure you want to delete "{ruleToDelete?.name}"? This
                action cannot be undone.
            </Dialog.Description>
        </Dialog.Header>
        <Dialog.Footer>
            <Button variant="outline" onclick={() => (deleteDialogOpen = false)}
                >Cancel</Button
            >
            <Button variant="destructive" onclick={deleteRule}>Delete</Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
