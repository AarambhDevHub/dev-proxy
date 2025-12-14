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
	import { RefreshCw, Trash2, Plus, Pencil, Wand2 } from "lucide-svelte";
	import type { ModifierRule } from "$lib/types";

	let rules: ModifierRule[] = $state([]);
	let loading = $state(true);
	let deleteDialogOpen = $state(false);
	let ruleToDelete: ModifierRule | null = $state(null);

	async function fetchRules() {
		try {
			const res = await fetch("/api/modifiers");
			rules = await res.json();
		} catch (error) {
			console.error("Failed to fetch modifier rules:", error);
			toast.error("Failed to fetch modifier rules");
		} finally {
			loading = false;
		}
	}

	async function toggleRule(id: string) {
		try {
			await fetch(`/api/modifiers/${id}/toggle`, { method: "POST" });
			await fetchRules();
			toast.success("Modifier rule toggled");
		} catch (error) {
			console.error("Failed to toggle rule:", error);
			toast.error("Failed to toggle rule");
		}
	}

	async function deleteRule() {
		if (!ruleToDelete) return;
		try {
			await fetch(`/api/modifiers/${ruleToDelete.id}`, {
				method: "DELETE",
			});
			await fetchRules();
			toast.success("Modifier rule deleted");
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
			await fetch("/api/modifiers", { method: "DELETE" });
			rules = [];
			toast.success("All modifier rules cleared");
		} catch (error) {
			console.error("Failed to clear rules:", error);
			toast.error("Failed to clear rules");
		}
	}

	function getMatchTypeLabel(type: string): string {
		switch (type) {
			case "exact":
				return "Exact";
			case "contains":
				return "Contains";
			case "regex":
				return "Regex";
			case "startswith":
				return "Starts With";
			case "endswith":
				return "Ends With";
			default:
				return type;
		}
	}

	function getModificationTypeLabel(type: string): string {
		switch (type) {
			case "replace_body":
				return "Replace Body";
			case "add_header":
				return "Add Header";
			case "remove_header":
				return "Remove Header";
			case "change_status":
				return "Change Status";
			case "inject_delay":
				return "Inject Delay";
			case "modify_json":
				return "Modify JSON";
			default:
				return type;
		}
	}

	function confirmDelete(rule: ModifierRule) {
		ruleToDelete = rule;
		deleteDialogOpen = true;
	}

	onMount(() => {
		fetchRules();
	});
</script>

<!-- Header -->
<div class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
	<div>
		<h1 class="text-3xl font-bold tracking-tight">Response Modifiers</h1>
		<p class="text-muted-foreground">
			Transform responses on-the-fly (MITM)
		</p>
	</div>
	<div class="flex items-center gap-2">
		<Button variant="outline" size="sm" onclick={fetchRules}>
			<RefreshCw class="mr-2 h-4 w-4" />
			Refresh
		</Button>
		{#if rules.length > 0}
			<Button variant="destructive" size="sm" onclick={clearAll}>
				<Trash2 class="mr-2 h-4 w-4" />
				Clear All
			</Button>
		{/if}
		<Button size="sm" onclick={() => goto("/modifiers/new")}>
			<Plus class="mr-2 h-4 w-4" />
			New Modifier
		</Button>
	</div>
</div>

{#if loading}
	<div class="grid gap-4 mt-6">
		{#each Array(3) as _}
			<Card.Root>
				<Card.Content class="p-6">
					<div class="flex items-center gap-4">
						<Skeleton class="h-6 w-32" />
						<Skeleton class="h-6 w-16" />
						<Skeleton class="h-6 flex-1" />
					</div>
				</Card.Content>
			</Card.Root>
		{/each}
	</div>
{:else if rules.length === 0}
	<Card.Root class="mt-6">
		<Card.Content class="flex flex-col items-center justify-center py-12">
			<Wand2 class="h-12 w-12 text-muted-foreground mb-4" />
			<h3 class="text-lg font-semibold">No Modifier Rules</h3>
			<p class="text-muted-foreground mb-4">
				Create modifiers to transform responses on-the-fly
			</p>
			<Button onclick={() => goto("/modifiers/new")}>
				<Plus class="mr-2 h-4 w-4" />
				Create Modifier
			</Button>
		</Card.Content>
	</Card.Root>
{:else}
	<div class="grid gap-4 mt-6">
		{#each rules as rule (rule.id)}
			<Card.Root class="hover:shadow-md transition-shadow">
				<Card.Content class="p-6">
					<div class="flex items-start justify-between gap-4">
						<div class="flex-1 min-w-0">
							<div class="flex items-center gap-3 mb-3 flex-wrap">
								<h3 class="text-lg font-semibold">
									{rule.name}
								</h3>
								<Badge
									variant={rule.enabled
										? "default"
										: "secondary"}
								>
									{rule.enabled ? "Enabled" : "Disabled"}
								</Badge>
								<Badge variant="outline"
									>Priority: {rule.priority}</Badge
								>
							</div>

							<div class="space-y-2 text-sm">
								<div class="flex items-center gap-2 flex-wrap">
									<span
										class="text-muted-foreground font-medium"
										>Match:</span
									>
									{#if rule.match_request.method}
										<Badge variant="outline"
											>{rule.match_request.method}</Badge
										>
									{/if}
									<code
										class="px-2 py-0.5 bg-muted rounded font-mono"
										>{rule.match_request.url_pattern}</code
									>
									<Badge variant="outline" class="text-xs"
										>{getMatchTypeLabel(
											rule.match_request.url_match_type,
										)}</Badge
									>
								</div>

								{#if rule.match_request.status_codes && rule.match_request.status_codes.length > 0}
									<div
										class="flex items-center gap-2 flex-wrap"
									>
										<span
											class="text-muted-foreground font-medium"
											>Status Codes:</span
										>
										{#each rule.match_request.status_codes as status}
											<Badge variant="outline"
												>{status}</Badge
											>
										{/each}
									</div>
								{/if}

								<div class="flex items-center gap-2 flex-wrap">
									<span
										class="text-muted-foreground font-medium"
										>Modifications:</span
									>
									{#each rule.modifications as mod}
										<Badge variant="secondary"
											>{getModificationTypeLabel(
												mod.type,
											)}</Badge
										>
									{/each}
								</div>

								<p class="text-xs text-muted-foreground">
									Created: {new Date(
										rule.created_at,
									).toLocaleString()}
								</p>
							</div>
						</div>

						<div class="flex items-center gap-2">
							<Switch
								checked={rule.enabled}
								onCheckedChange={() => toggleRule(rule.id)}
							/>
							<Button
								variant="ghost"
								size="icon"
								onclick={() => goto(`/modifiers/${rule.id}`)}
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
					</div>
				</Card.Content>
			</Card.Root>
		{/each}
	</div>
{/if}

<!-- Delete Confirmation Dialog -->
<Dialog.Root bind:open={deleteDialogOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Delete Modifier Rule</Dialog.Title>
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
