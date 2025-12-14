<script lang="ts">
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Badge } from "$lib/components/ui/badge";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Separator } from "$lib/components/ui/separator";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import {
    RefreshCw,
    Trash2,
    Search,
    Clock,
    CheckCircle,
    XCircle,
    AlertTriangle,
    ArrowUpDown,
    ExternalLink,
  } from "lucide-svelte";
  import type {
    RecordedRequest,
    RecordingStats,
    FilterOptions,
  } from "$lib/types";
  import {
    formatBody,
    formatDuration,
    getStatusColor,
    getMethodColor,
  } from "$lib/types";

  let recordings: RecordedRequest[] = $state([]);
  let stats: RecordingStats | null = $state(null);
  let loading = $state(true);
  let autoRefresh = $state(true);
  let refreshInterval: number;
  let selectedRequest: RecordedRequest | null = $state(null);
  let detailOpen = $state(false);

  let filters: FilterOptions = $state({
    search: "",
    method: "",
    status: undefined,
    minDuration: undefined,
    maxDuration: undefined,
  });

  const methods = [
    "",
    "GET",
    "POST",
    "PUT",
    "DELETE",
    "PATCH",
    "HEAD",
    "OPTIONS",
  ];

  async function fetchRecordings() {
    try {
      const params = new URLSearchParams();
      if (filters.search) params.append("search", filters.search);
      if (filters.method) params.append("method", filters.method);
      if (filters.status) params.append("status", filters.status.toString());
      if (filters.minDuration)
        params.append("minDuration", filters.minDuration.toString());
      if (filters.maxDuration)
        params.append("maxDuration", filters.maxDuration.toString());

      const queryString = params.toString();
      const url = `/api/recordings${queryString ? "?" + queryString : ""}`;
      const res = await fetch(url);
      recordings = await res.json();
    } catch (error) {
      console.error("Failed to fetch recordings:", error);
      toast.error("Failed to fetch recordings");
    } finally {
      loading = false;
    }
  }

  async function fetchStats() {
    try {
      const res = await fetch("/api/stats");
      stats = await res.json();
    } catch (error) {
      console.error("Failed to fetch stats:", error);
    }
  }

  async function clearRecordings() {
    try {
      await fetch("/api/recordings", { method: "DELETE" });
      recordings = [];
      stats = null;
      await fetchStats();
      toast.success("All recordings cleared");
    } catch (error) {
      console.error("Failed to clear recordings:", error);
      toast.error("Failed to clear recordings");
    }
  }

  function openDetail(request: RecordedRequest) {
    selectedRequest = request;
    detailOpen = true;
  }

  function getMethodBadgeVariant(
    method: string,
  ): "default" | "secondary" | "destructive" | "outline" {
    switch (method) {
      case "GET":
        return "secondary";
      case "POST":
        return "default";
      case "DELETE":
        return "destructive";
      default:
        return "outline";
    }
  }

  function getStatusBadgeVariant(
    status?: number,
  ): "default" | "secondary" | "destructive" | "outline" {
    if (!status) return "outline";
    if (status >= 200 && status < 300) return "default";
    if (status >= 400) return "destructive";
    return "secondary";
  }

  onMount(() => {
    fetchRecordings();
    fetchStats();

    if (autoRefresh) {
      refreshInterval = setInterval(() => {
        fetchRecordings();
        fetchStats();
      }, 2000);
    }

    return () => {
      if (refreshInterval) clearInterval(refreshInterval);
    };
  });

  $effect(() => {
    if (autoRefresh) {
      refreshInterval = setInterval(() => {
        fetchRecordings();
        fetchStats();
      }, 2000);
    } else {
      if (refreshInterval) clearInterval(refreshInterval);
    }
  });

  $effect(() => {
    // Debounced filter change
    const timeout = setTimeout(() => {
      fetchRecordings();
    }, 300);
    return () => clearTimeout(timeout);
  });
</script>

<!-- Header -->
<div class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
  <div>
    <h1 class="text-3xl font-bold tracking-tight">Traffic Recordings</h1>
    <p class="text-muted-foreground">
      Monitor and inspect HTTP traffic passing through the proxy
    </p>
  </div>
  <div class="flex items-center gap-2">
    <div class="flex items-center space-x-2">
      <Checkbox id="auto-refresh" bind:checked={autoRefresh} />
      <Label for="auto-refresh" class="text-sm font-medium">Auto-refresh</Label>
    </div>
    <Button
      variant="outline"
      size="sm"
      onclick={() => {
        fetchRecordings();
        fetchStats();
      }}
    >
      <RefreshCw class="mr-2 h-4 w-4" />
      Refresh
    </Button>
    <Button variant="destructive" size="sm" onclick={clearRecordings}>
      <Trash2 class="mr-2 h-4 w-4" />
      Clear All
    </Button>
  </div>
</div>

<!-- Stats Cards -->
{#if stats}
  <div class="grid gap-4 md:grid-cols-4 mt-6">
    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">Total Requests</Card.Title>
        <ArrowUpDown class="h-4 w-4 text-muted-foreground" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">{stats.total}</div>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">Success Rate</Card.Title>
        <CheckCircle class="h-4 w-4 text-green-500" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold text-green-600">
          {stats.total > 0
            ? ((stats.success / stats.total) * 100).toFixed(1)
            : 0}%
        </div>
        <p class="text-xs text-muted-foreground">{stats.success} successful</p>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">Errors</Card.Title>
        <XCircle class="h-4 w-4 text-red-500" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold text-red-600">
          {stats.client_errors + stats.server_errors}
        </div>
        <p class="text-xs text-muted-foreground">
          {stats.client_errors} client, {stats.server_errors} server
        </p>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">Avg Duration</Card.Title>
        <Clock class="h-4 w-4 text-muted-foreground" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">
          {formatDuration(stats.avg_duration_ms)}
        </div>
        <p class="text-xs text-muted-foreground">
          {formatDuration(stats.min_duration_ms)} - {formatDuration(
            stats.max_duration_ms,
          )}
        </p>
      </Card.Content>
    </Card.Root>
  </div>
{/if}

<!-- Filters -->
<Card.Root class="mt-6">
  <Card.Header>
    <Card.Title class="text-lg">Filters</Card.Title>
  </Card.Header>
  <Card.Content>
    <div class="flex flex-wrap gap-4">
      <div class="flex-1 min-w-[200px]">
        <div class="relative">
          <Search
            class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground"
          />
          <Input
            placeholder="Search URL or body..."
            class="pl-8"
            bind:value={filters.search}
          />
        </div>
      </div>
      <div class="w-[130px]">
        <Select.Root
          type="single"
          value={filters.method
            ? { value: filters.method, label: filters.method }
            : undefined}
          onValueChange={(v) => (filters.method = v?.value || "")}
        >
          <Select.Trigger>
            {filters.method || "All Methods"}
          </Select.Trigger>
          <Select.Content>
            {#each methods as method}
              <Select.Item value={method}>{method || "All Methods"}</Select.Item
              >
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
      <div class="w-[120px]">
        <Input type="number" placeholder="Status" bind:value={filters.status} />
      </div>
      <div class="w-[120px]">
        <Input
          type="number"
          placeholder="Min ms"
          bind:value={filters.minDuration}
        />
      </div>
      <div class="w-[120px]">
        <Input
          type="number"
          placeholder="Max ms"
          bind:value={filters.maxDuration}
        />
      </div>
    </div>
  </Card.Content>
</Card.Root>

<!-- Recordings List -->
<Card.Root class="mt-6">
  <Card.Header>
    <Card.Title class="text-lg">Requests ({recordings.length})</Card.Title>
  </Card.Header>
  <Card.Content class="p-0">
    {#if loading}
      <div class="p-6 space-y-4">
        {#each Array(5) as _}
          <div class="flex items-center space-x-4">
            <Skeleton class="h-6 w-16" />
            <Skeleton class="h-6 flex-1" />
            <Skeleton class="h-6 w-12" />
            <Skeleton class="h-6 w-20" />
          </div>
        {/each}
      </div>
    {:else if recordings.length === 0}
      <div class="flex flex-col items-center justify-center py-12 text-center">
        <AlertTriangle class="h-12 w-12 text-muted-foreground mb-4" />
        <h3 class="text-lg font-semibold">No recordings found</h3>
        <p class="text-muted-foreground">
          {filters.search || filters.method || filters.status
            ? "No recordings match your filters."
            : "Send some requests through the proxy to see them here!"}
        </p>
      </div>
    {:else}
      <div class="divide-y">
        {#each recordings as request (request.id)}
          <button
            class="w-full flex items-center gap-4 p-4 hover:bg-muted/50 transition-colors text-left"
            onclick={() => openDetail(request)}
          >
            <Badge
              variant={getMethodBadgeVariant(request.method)}
              class="w-16 justify-center"
            >
              {request.method}
            </Badge>
            <div class="flex-1 min-w-0">
              <p class="font-mono text-sm truncate">{request.url}</p>
              <p class="text-xs text-muted-foreground">
                {new Date(request.timestamp).toLocaleTimeString()}
              </p>
            </div>
            <Badge variant={getStatusBadgeVariant(request.response?.status)}>
              {request.response?.status || "Pending"}
            </Badge>
            <span class="text-sm text-muted-foreground w-20 text-right">
              {formatDuration(request.duration_ms)}
            </span>
            <ExternalLink class="h-4 w-4 text-muted-foreground" />
          </button>
        {/each}
      </div>
    {/if}
  </Card.Content>
</Card.Root>

<!-- Request Detail Dialog -->
<Dialog.Root bind:open={detailOpen}>
  <Dialog.Content class="max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
    <Dialog.Header>
      <Dialog.Title class="flex items-center gap-2">
        {#if selectedRequest}
          <Badge variant={getMethodBadgeVariant(selectedRequest.method)}>
            {selectedRequest.method}
          </Badge>
          <span class="font-mono text-sm truncate">{selectedRequest.url}</span>
        {/if}
      </Dialog.Title>
      <Dialog.Description>
        {#if selectedRequest}
          {new Date(selectedRequest.timestamp).toLocaleString()} •
          {formatDuration(selectedRequest.duration_ms)}
        {/if}
      </Dialog.Description>
    </Dialog.Header>

    {#if selectedRequest}
      <ScrollArea class="flex-1 pr-4">
        <div class="space-y-6">
          <!-- Request Headers -->
          <div>
            <h4 class="font-semibold mb-2">Request Headers</h4>
            <div class="bg-muted rounded-lg p-4 font-mono text-sm">
              {#each Object.entries(selectedRequest.headers) as [key, value]}
                <div>
                  <span class="text-muted-foreground">{key}:</span>
                  {value}
                </div>
              {/each}
            </div>
          </div>

          <!-- Request Body -->
          {#if selectedRequest.body && selectedRequest.body.length > 0}
            <div>
              <h4 class="font-semibold mb-2">Request Body</h4>
              <pre
                class="bg-muted rounded-lg p-4 font-mono text-sm overflow-x-auto whitespace-pre-wrap">{formatBody(
                  selectedRequest.body,
                )}</pre>
            </div>
          {/if}

          <Separator />

          <!-- Response -->
          {#if selectedRequest.response}
            <div>
              <h4 class="font-semibold mb-2 flex items-center gap-2">
                Response
                <Badge
                  variant={getStatusBadgeVariant(
                    selectedRequest.response.status,
                  )}
                >
                  {selectedRequest.response.status}
                </Badge>
              </h4>
            </div>

            <!-- Response Headers -->
            <div>
              <h4 class="font-semibold mb-2">Response Headers</h4>
              <div class="bg-muted rounded-lg p-4 font-mono text-sm">
                {#each Object.entries(selectedRequest.response.headers) as [key, value]}
                  <div>
                    <span class="text-muted-foreground">{key}:</span>
                    {value}
                  </div>
                {/each}
              </div>
            </div>

            <!-- Response Body -->
            {#if selectedRequest.response.body && selectedRequest.response.body.length > 0}
              <div>
                <h4 class="font-semibold mb-2">Response Body</h4>
                <pre
                  class="bg-muted rounded-lg p-4 font-mono text-sm overflow-x-auto whitespace-pre-wrap max-h-96">{formatBody(
                    selectedRequest.response.body,
                  )}</pre>
              </div>
            {/if}
          {/if}
        </div>
      </ScrollArea>
    {/if}
  </Dialog.Content>
</Dialog.Root>
