<script lang="ts">
  import { onMount } from "svelte";
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Label } from "$lib/components/ui/label";
  import { Separator } from "$lib/components/ui/separator";
  import {
    RefreshCw,
    TrendingUp,
    Clock,
    CheckCircle,
    XCircle,
    BarChart3,
    Activity,
  } from "lucide-svelte";
  import type {
    Analytics,
    RecordingStats,
    EndpointStats,
    TimelinePoint,
  } from "$lib/types";
  import { formatDuration } from "$lib/types";

  let stats: RecordingStats | null = $state(null);
  let analytics: Analytics | null = $state(null);
  let loading = $state(true);
  let autoRefresh = $state(true);
  let refreshInterval: number;

  async function fetchData() {
    try {
      const [statsRes, analyticsRes] = await Promise.all([
        fetch("/api/stats"),
        fetch("/api/analytics"),
      ]);
      stats = await statsRes.json();
      analytics = await analyticsRes.json();
    } catch (error) {
      console.error("Failed to fetch dashboard data:", error);
    } finally {
      loading = false;
    }
  }

  function getMethodColor(method: string): string {
    switch (method) {
      case "GET":
        return "bg-blue-500";
      case "POST":
        return "bg-green-500";
      case "PUT":
        return "bg-yellow-500";
      case "DELETE":
        return "bg-red-500";
      case "PATCH":
        return "bg-purple-500";
      default:
        return "bg-gray-500";
    }
  }

  function getStatusColor(status: number): string {
    if (status >= 200 && status < 300) return "bg-green-500";
    if (status >= 300 && status < 400) return "bg-blue-500";
    if (status >= 400 && status < 500) return "bg-yellow-500";
    return "bg-red-500";
  }

  onMount(() => {
    fetchData();

    if (autoRefresh) {
      refreshInterval = setInterval(fetchData, 3000);
    }

    return () => {
      if (refreshInterval) clearInterval(refreshInterval);
    };
  });

  $effect(() => {
    if (autoRefresh) {
      refreshInterval = setInterval(fetchData, 3000);
    } else {
      if (refreshInterval) clearInterval(refreshInterval);
    }
  });
</script>

<!-- Header -->
<div class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
  <div>
    <h1 class="text-3xl font-bold tracking-tight">Traffic Dashboard</h1>
    <p class="text-muted-foreground">
      Analytics and insights from your proxy traffic
    </p>
  </div>
  <div class="flex items-center gap-2">
    <div class="flex items-center space-x-2">
      <Checkbox id="auto-refresh" bind:checked={autoRefresh} />
      <Label for="auto-refresh" class="text-sm font-medium">Auto-refresh</Label>
    </div>
    <Button variant="outline" size="sm" onclick={fetchData}>
      <RefreshCw class="mr-2 h-4 w-4" />
      Refresh
    </Button>
  </div>
</div>

{#if loading}
  <div class="grid gap-4 md:grid-cols-4 mt-6">
    {#each Array(4) as _}
      <Card.Root>
        <Card.Header class="pb-2">
          <Skeleton class="h-4 w-24" />
        </Card.Header>
        <Card.Content>
          <Skeleton class="h-8 w-16" />
        </Card.Content>
      </Card.Root>
    {/each}
  </div>
{:else if stats && analytics}
  <!-- Stats Overview -->
  <div class="grid gap-4 md:grid-cols-4 mt-6">
    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">Total Requests</Card.Title>
        <Activity class="h-4 w-4 text-muted-foreground" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">{stats.total.toLocaleString()}</div>
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
      </Card.Content>
    </Card.Root>
  </div>

  <!-- Charts Row -->
  <div class="grid gap-6 md:grid-cols-2 mt-6">
    <!-- Method Distribution -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <BarChart3 class="h-5 w-5" />
          HTTP Methods
        </Card.Title>
        <Card.Description>Distribution of request methods</Card.Description>
      </Card.Header>
      <Card.Content>
        {#if Object.keys(analytics.method_distribution).length > 0}
          <div class="space-y-3">
            {#each Object.entries(analytics.method_distribution) as [method, count]}
              {@const total = Object.values(
                analytics.method_distribution,
              ).reduce((a, b) => a + b, 0)}
              {@const percentage = total > 0 ? (count / total) * 100 : 0}
              <div class="flex items-center gap-3">
                <div class="w-16 font-medium text-sm">{method}</div>
                <div class="flex-1 h-4 bg-muted rounded-full overflow-hidden">
                  <div
                    class="{getMethodColor(
                      method,
                    )} h-full transition-all duration-300"
                    style="width: {percentage}%"
                  ></div>
                </div>
                <div class="w-16 text-sm text-muted-foreground text-right">
                  {count}
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="text-muted-foreground text-center py-8">
            No data available
          </p>
        {/if}
      </Card.Content>
    </Card.Root>

    <!-- Status Distribution -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <TrendingUp class="h-5 w-5" />
          Status Codes
        </Card.Title>
        <Card.Description>Response status distribution</Card.Description>
      </Card.Header>
      <Card.Content>
        {#if Object.keys(analytics.status_distribution).length > 0}
          <div class="space-y-3">
            {#each Object.entries(analytics.status_distribution).sort((a, b) => parseInt(a[0]) - parseInt(b[0])) as [status, count]}
              {@const total = Object.values(
                analytics.status_distribution,
              ).reduce((a, b) => a + b, 0)}
              {@const percentage = total > 0 ? (count / total) * 100 : 0}
              <div class="flex items-center gap-3">
                <Badge
                  variant={parseInt(status) >= 400 ? "destructive" : "default"}
                  class="w-16 justify-center"
                >
                  {status}
                </Badge>
                <div class="flex-1 h-4 bg-muted rounded-full overflow-hidden">
                  <div
                    class="{getStatusColor(
                      parseInt(status),
                    )} h-full transition-all duration-300"
                    style="width: {percentage}%"
                  ></div>
                </div>
                <div class="w-16 text-sm text-muted-foreground text-right">
                  {count}
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="text-muted-foreground text-center py-8">
            No data available
          </p>
        {/if}
      </Card.Content>
    </Card.Root>
  </div>

  <!-- Timeline -->
  <Card.Root class="mt-6">
    <Card.Header>
      <Card.Title class="flex items-center gap-2">
        <Activity class="h-5 w-5" />
        Request Timeline
      </Card.Title>
      <Card.Description>Recent requests over time</Card.Description>
    </Card.Header>
    <Card.Content>
      {#if analytics.timeline.length > 0}
        <div class="flex items-end gap-1 h-32">
          {#each analytics.timeline.slice(-60) as point}
            {@const maxDuration = Math.max(
              ...analytics.timeline.map((p) => p.duration_ms),
              1,
            )}
            {@const height = Math.max(
              (point.duration_ms / maxDuration) * 100,
              5,
            )}
            <div
              class="flex-1 rounded-t transition-all duration-200 hover:opacity-80 {getStatusColor(
                point.status,
              )}"
              style="height: {height}%"
              title="{point.method} {point.status} - {point.duration_ms}ms"
            ></div>
          {/each}
        </div>
        <div class="flex justify-between text-xs text-muted-foreground mt-2">
          <span>Older</span>
          <span>Recent</span>
        </div>
      {:else}
        <p class="text-muted-foreground text-center py-8">
          No timeline data available
        </p>
      {/if}
    </Card.Content>
  </Card.Root>

  <!-- Top Endpoints -->
  <Card.Root class="mt-6">
    <Card.Header>
      <Card.Title>Top Endpoints</Card.Title>
      <Card.Description>Most frequently accessed endpoints</Card.Description>
    </Card.Header>
    <Card.Content>
      {#if analytics.top_endpoints.length > 0}
        <div class="space-y-4">
          {#each analytics.top_endpoints.slice(0, 10) as endpoint}
            {@const maxCount = Math.max(
              ...analytics.top_endpoints.map((e) => e.count),
              1,
            )}
            {@const percentage = (endpoint.count / maxCount) * 100}
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <code class="text-sm font-mono truncate max-w-md"
                  >{endpoint.endpoint}</code
                >
                <div
                  class="flex items-center gap-4 text-sm text-muted-foreground"
                >
                  <span>{endpoint.count} requests</span>
                  <span>{formatDuration(endpoint.avg_duration)} avg</span>
                  {#if endpoint.errors > 0}
                    <Badge variant="destructive">{endpoint.errors} errors</Badge
                    >
                  {/if}
                </div>
              </div>
              <div class="h-2 bg-muted rounded-full overflow-hidden">
                <div
                  class="bg-primary h-full transition-all duration-300"
                  style="width: {percentage}%"
                ></div>
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-muted-foreground text-center py-8">
          No endpoint data available
        </p>
      {/if}
    </Card.Content>
  </Card.Root>
{:else}
  <Card.Root class="mt-6">
    <Card.Content class="flex flex-col items-center justify-center py-12">
      <BarChart3 class="h-12 w-12 text-muted-foreground mb-4" />
      <h3 class="text-lg font-semibold">No Data Available</h3>
      <p class="text-muted-foreground">
        Start recording some traffic to see analytics!
      </p>
    </Card.Content>
  </Card.Root>
{/if}
