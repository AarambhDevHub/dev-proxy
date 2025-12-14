<script lang="ts">
  import { goto } from "$app/navigation";
  import { toast } from "svelte-sonner";
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Textarea } from "$lib/components/ui/textarea";
  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";
  import * as Select from "$lib/components/ui/select";
  import { Separator } from "$lib/components/ui/separator";
  import * as Alert from "$lib/components/ui/alert";
  import { ArrowLeft, Save, Code, AlertCircle } from "lucide-svelte";
  import type { CreateMockRule } from "$lib/types";

  let rule: CreateMockRule = $state({
    name: "",
    enabled: true,
    priority: 0,
    method: "",
    url_pattern: "",
    url_match_type: "contains",
    response: {
      status: 200,
      headers: { "content-type": "application/json" },
      body: '{"message": "Mocked response"}',
    },
    delay_ms: undefined,
  });

  let headersText = $state("content-type: application/json");
  let saving = $state(false);
  let error = $state("");

  const methods = [
    { value: "", label: "Any Method" },
    { value: "GET", label: "GET" },
    { value: "POST", label: "POST" },
    { value: "PUT", label: "PUT" },
    { value: "PATCH", label: "PATCH" },
    { value: "DELETE", label: "DELETE" },
  ];

  const matchTypes = [
    { value: "exact", label: "Exact Match" },
    { value: "contains", label: "Contains" },
    { value: "startswith", label: "Starts With" },
    { value: "endswith", label: "Ends With" },
    { value: "regex", label: "Regex" },
  ];

  async function save() {
    if (!rule.name || !rule.url_pattern) {
      error = "Name and URL pattern are required";
      return;
    }

    saving = true;
    error = "";

    try {
      const headers: Record<string, string> = {};
      headersText.split("\n").forEach((line) => {
        const [key, ...valueParts] = line.split(":");
        if (key && valueParts.length > 0) {
          headers[key.trim()] = valueParts.join(":").trim();
        }
      });

      const payload = {
        ...rule,
        method: rule.method || undefined,
        response: {
          ...rule.response,
          headers,
        },
      };

      const res = await fetch("/api/mocks", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });

      if (res.ok) {
        toast.success("Mock rule created");
        goto("/mocks");
      } else {
        error = "Failed to create mock rule";
      }
    } catch (e) {
      error = "Failed to create mock rule: " + e;
    } finally {
      saving = false;
    }
  }

  function formatJSON() {
    try {
      const parsed = JSON.parse(rule.response!.body);
      rule.response!.body = JSON.stringify(parsed, null, 2);
    } catch {
      // Invalid JSON
    }
  }
</script>

<div class="max-w-4xl mx-auto">
  <div class="flex items-center gap-4 mb-6">
    <Button variant="ghost" size="sm" onclick={() => goto("/mocks")}>
      <ArrowLeft class="mr-2 h-4 w-4" />
      Back
    </Button>
    <h1 class="text-3xl font-bold tracking-tight">Create Mock Rule</h1>
  </div>

  {#if error}
    <Alert.Root variant="destructive" class="mb-6">
      <AlertCircle class="h-4 w-4" />
      <Alert.Title>Error</Alert.Title>
      <Alert.Description>{error}</Alert.Description>
    </Alert.Root>
  {/if}

  <Card.Root>
    <Card.Content class="pt-6 space-y-6">
      <!-- Basic Info -->
      <div class="grid grid-cols-2 gap-4">
        <div class="space-y-2">
          <Label for="name">Rule Name *</Label>
          <Input
            id="name"
            bind:value={rule.name}
            placeholder="e.g., Mock User API"
          />
        </div>
        <div class="space-y-2">
          <Label for="priority">Priority</Label>
          <Input id="priority" type="number" bind:value={rule.priority} />
          <p class="text-xs text-muted-foreground">
            Higher priority rules match first
          </p>
        </div>
      </div>

      <Separator />

      <!-- Request Matching -->
      <div>
        <h3 class="text-lg font-semibold mb-4">Request Matching</h3>
        <div class="grid grid-cols-2 gap-4 mb-4">
          <div class="space-y-2">
            <Label>HTTP Method</Label>
            <Select.Root
              type="single"
              value={methods.find((m) => m.value === rule.method)}
              onValueChange={(v) => (rule.method = v?.value || "")}
            >
              <Select.Trigger>{rule.method || "Any Method"}</Select.Trigger>
              <Select.Content>
                {#each methods as method}
                  <Select.Item value={method.value}>{method.label}</Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
          </div>
          <div class="space-y-2">
            <Label>Match Type</Label>
            <Select.Root
              type="single"
              value={matchTypes.find((m) => m.value === rule.url_match_type)}
              onValueChange={(v) =>
                (rule.url_match_type = (v?.value as any) || "contains")}
            >
              <Select.Trigger
                >{matchTypes.find((m) => m.value === rule.url_match_type)
                  ?.label}</Select.Trigger
              >
              <Select.Content>
                {#each matchTypes as type}
                  <Select.Item value={type.value}>{type.label}</Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
          </div>
        </div>
        <div class="space-y-2">
          <Label for="pattern">URL Pattern *</Label>
          <Input
            id="pattern"
            bind:value={rule.url_pattern}
            placeholder="e.g., /api/users"
          />
        </div>
      </div>

      <Separator />

      <!-- Mock Response -->
      <div>
        <h3 class="text-lg font-semibold mb-4">Mock Response</h3>
        <div class="grid grid-cols-2 gap-4 mb-4">
          <div class="space-y-2">
            <Label for="status">Status Code</Label>
            <Input
              id="status"
              type="number"
              bind:value={rule.response.status}
              min="100"
              max="599"
            />
          </div>
          <div class="space-y-2">
            <Label for="delay">Delay (ms)</Label>
            <Input
              id="delay"
              type="number"
              bind:value={rule.delay_ms}
              min="0"
              placeholder="No delay"
            />
          </div>
        </div>

        <div class="space-y-2 mb-4">
          <Label for="headers"
            >Response Headers (one per line, key: value)</Label
          >
          <Textarea
            id="headers"
            bind:value={headersText}
            rows={3}
            class="font-mono text-sm"
          />
        </div>

        <div class="space-y-2">
          <div class="flex justify-between items-center">
            <Label for="body">Response Body</Label>
            <Button variant="ghost" size="sm" onclick={formatJSON}>
              <Code class="mr-2 h-4 w-4" />
              Format JSON
            </Button>
          </div>
          <Textarea
            id="body"
            bind:value={rule.response.body}
            rows={10}
            class="font-mono text-sm"
          />
        </div>
      </div>

      <Separator />

      <!-- Options -->
      <div class="flex items-center space-x-2">
        <Switch id="enabled" bind:checked={rule.enabled} />
        <Label for="enabled">Enable this rule immediately</Label>
      </div>

      <Separator />

      <!-- Actions -->
      <div class="flex justify-end gap-4">
        <Button variant="outline" onclick={() => goto("/mocks")}>Cancel</Button>
        <Button onclick={save} disabled={saving}>
          <Save class="mr-2 h-4 w-4" />
          {saving ? "Creating..." : "Create Mock Rule"}
        </Button>
      </div>
    </Card.Content>
  </Card.Root>
</div>
