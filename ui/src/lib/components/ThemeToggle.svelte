<script lang="ts">
	import { Sun, Moon } from "lucide-svelte";
	import { Button } from "$lib/components/ui/button";

	let isDark = $state(false);

	function toggleTheme() {
		isDark = !isDark;
		if (isDark) {
			document.documentElement.classList.add("dark");
			localStorage.setItem("theme", "dark");
		} else {
			document.documentElement.classList.remove("dark");
			localStorage.setItem("theme", "light");
		}
	}

	$effect(() => {
		// Check initial theme
		const stored = localStorage.getItem("theme");
		const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
		isDark = stored === "dark" || (!stored && prefersDark);
		if (isDark) {
			document.documentElement.classList.add("dark");
		}
	});
</script>

<Button variant="ghost" size="icon" onclick={toggleTheme} class="h-9 w-9">
	{#if isDark}
		<Sun class="h-4 w-4" />
	{:else}
		<Moon class="h-4 w-4" />
	{/if}
	<span class="sr-only">Toggle theme</span>
</Button>
