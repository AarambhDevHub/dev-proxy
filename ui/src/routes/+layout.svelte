<script lang="ts">
    import "../app.css";
    import { page } from "$app/stores";
    import { goto } from "$app/navigation";
    import * as Tabs from "$lib/components/ui/tabs";
    import { Button } from "$lib/components/ui/button";
    import ThemeToggle from "$lib/components/ThemeToggle.svelte";
    import { Toaster } from "$lib/components/ui/sonner";
    import {
        Activity,
        BarChart3,
        Drama,
        Wand2,
        Timer,
        Gauge,
    } from "lucide-svelte";

    const tabs = [
        { value: "/", label: "Recordings", icon: Activity },
        { value: "/dashboard", label: "Dashboard", icon: BarChart3 },
        { value: "/mocks", label: "Mocks", icon: Drama },
        { value: "/modifiers", label: "Modifiers", icon: Wand2 },
        { value: "/rate-limits", label: "Rate Limits", icon: Gauge },
        { value: "/latency", label: "Latency", icon: Timer },
    ];

    function getCurrentTab(pathname: string): string {
        if (pathname === "/" || pathname === "") return "/";
        const match = tabs.find(
            (t) => t.value !== "/" && pathname.startsWith(t.value),
        );
        return match?.value || "/";
    }

    function handleTabChange(value: string) {
        goto(value);
    }
</script>

<div class="min-h-screen bg-background">
    <!-- Header -->
    <header
        class="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60"
    >
        <div class="container flex h-14 items-center">
            <div class="mr-4 flex">
                <a href="/" class="mr-6 flex items-center space-x-2">
                    <Activity class="h-6 w-6 text-primary" />
                    <span class="font-bold text-xl">Dev Proxy</span>
                </a>
            </div>
            <div class="flex flex-1 items-center justify-end space-x-2">
                <ThemeToggle />
            </div>
        </div>
    </header>

    <!-- Navigation Tabs -->
    <div class="border-b bg-background">
        <div class="container">
            <Tabs.Root
                value={getCurrentTab($page.url.pathname)}
                onValueChange={handleTabChange}
            >
                <Tabs.List
                    class="h-12 w-full justify-start rounded-none border-0 bg-transparent p-0"
                >
                    {#each tabs as tab (tab.value)}
                        <Tabs.Trigger
                            value={tab.value}
                            class="relative h-12 rounded-none border-b-2 border-b-transparent bg-transparent px-4 pb-3 pt-2 font-medium text-muted-foreground shadow-none transition-none data-[state=active]:border-b-primary data-[state=active]:text-foreground data-[state=active]:shadow-none"
                        >
                            <tab.icon class="mr-2 h-4 w-4" />
                            {tab.label}
                        </Tabs.Trigger>
                    {/each}
                </Tabs.List>
            </Tabs.Root>
        </div>
    </div>

    <!-- Main Content -->
    <main class="container py-6">
        <slot />
    </main>

    <Toaster />
</div>
