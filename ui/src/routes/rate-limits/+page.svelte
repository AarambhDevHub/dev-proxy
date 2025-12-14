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
        Gauge,
        RotateCcw,
    } from "lucide-svelte";
    import type { RateLimitRule, BucketStats } from "$lib/types";

    let rules: RateLimitRule[] = $state([]);
    let stats: BucketStats | null = $state(null);
    let loading = $state(true);
    let deleteDialogOpen = $state(false);
    let ruleToDelete: RateLimitRule | null = $state(null);

    async function fetchRules() {
        try {
            const res = await fetch("/api/rate-limits");
            rules = await res.json();
        } catch (error) {
            console.error("Failed to fetch rate limit rules:", error);
            toast.error("Failed to fetch rate limit rules");
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
            toast.success("Rate limit rule toggled");
        } catch (error) {
            console.error("Failed to toggle rule:", error);
            toast.error("Failed to toggle rule");
        }
    }

    async function resetBucket(id: string) {
        try {
            await fetch(`/api/rate-limits/${id}/reset`, { method: "POST" });
            await fetchStats();
            toast.success("Rate limit counters reset");
        } catch (error) {
            console.error("Failed to reset bucket:", error);
            toast.error("Failed to reset bucket");
        }
    }

    async function deleteRule() {
        if (!ruleToDelete) return;
        try {
            await fetch(`/api/rate-limits/${ruleToDelete.id}`, {
                method: "DELETE",
            });
            await fetchRules();
            await fetchStats();
            toast.success("Rate limit rule deleted");
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
            await fetch("/api/rate-limits", { method: "DELETE" });
            rules = [];
            stats = null;
            await fetchStats();
            toast.success("All rate limit rules cleared");
        } catch (error) {
            console.error("Failed to clear rules:", error);
            toast.error("Failed to clear rules");
        }
    }

    function getKeyTypeLabel(keyType: any): string {
        if (keyType === "global") return "Global";
        if (keyType === "ipaddress") return "IP Address";
        if (typeof keyType === "object" && "header" in keyType)
            return `Header: ${keyType.header.name}`;
        if (typeof keyType === "object" && "custom" in keyType)
            return `Custom: ${keyType.custom.pattern}`;
        return "Unknown";
    }

    function confirmDelete(rule: RateLimitRule) {
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
        <h1 class="text-3xl font-bold tracking-tight">Rate Limit Simulation</h1>
        <p class="text-muted-foreground">
            Test rate limiting behavior for your applications
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
        {#if rules.length > 0}
            <Button variant="destructive" size="sm" onclick={clearAll}>
                <Trash2 class="mr-2 h-4 w-4" />
                Clear All
            </Button>
        {/if}
        <Button size="sm" onclick={() => goto("/rate-limits/new")}>
            <Plus class="mr-2 h-4 w-4" />
            New Rule
        </Button>
    </div>
</div>

<!-- Stats -->
{#if stats}
    <div class="grid gap-4 md:grid-cols-2 mt-6">
        <Card.Root>
            <Card.Header
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <Card.Title class="text-sm font-medium"
                    >Active Buckets</Card.Title
                >
                <Gauge class="h-4 w-4 text-muted-foreground" />
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold text-blue-600">
                    {stats.active_limits}
                </div>
            </Card.Content>
        </Card.Root>

        <Card.Root>
            <Card.Header
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <Card.Title class="text-sm font-medium"
                    >Total Trackers</Card.Title
                >
                <Gauge class="h-4 w-4 text-muted-foreground" />
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold text-purple-600">
                    {stats.total_buckets}
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
            <Gauge class="h-12 w-12 text-muted-foreground mb-4" />
            <h3 class="text-lg font-semibold">No Rate Limit Rules</h3>
            <p class="text-muted-foreground mb-4">
                Get started by creating a new rate limit rule
            </p>
            <Button onclick={() => goto("/rate-limits/new")}>
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
                    <Table.Head>Limit</Table.Head>
                    <Table.Head>Key Type</Table.Head>
                    <Table.Head>Priority</Table.Head>
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
                                {rule.limit.max_requests} req / {rule.limit
                                    .window_seconds}s
                            </div>
                            {#if rule.limit.burst_size}
                                <Badge variant="secondary" class="text-xs"
                                    >Burst: {rule.limit.burst_size}</Badge
                                >
                            {/if}
                        </Table.Cell>
                        <Table.Cell>
                            <span class="text-sm"
                                >{getKeyTypeLabel(
                                    rule.match_request.key_type,
                                )}</span
                            >
                        </Table.Cell>
                        <Table.Cell>{rule.priority}</Table.Cell>
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
                                    onclick={() => resetBucket(rule.id)}
                                >
                                    <RotateCcw class="h-4 w-4" />
                                </Button>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    onclick={() =>
                                        goto(`/rate-limits/${rule.id}`)}
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
            <Dialog.Title>Delete Rate Limit Rule</Dialog.Title>
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
