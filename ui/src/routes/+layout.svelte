<script lang="ts">
    import "../app.css";
    import Header from "$lib/components/Header.svelte";
    import { page } from "$app/stores";
</script>

<div class="min-h-screen flex flex-col bg-gray-50">
    <Header />

    <nav class="bg-white shadow-sm border-b">
        <div class="container mx-auto px-4">
            <div class="flex gap-1">
                <a
                    href="/"
                    class="px-6 py-3 font-medium border-b-2 {$page.url
                        .pathname === '/'
                        ? 'border-blue-600 text-blue-600'
                        : 'border-transparent text-gray-600 hover:text-gray-900'}"
                >
                    Recordings
                </a>
                <a
                    href="/dashboard"
                    class="px-6 py-3 font-medium border-b-2 {$page.url
                        .pathname === '/dashboard'
                        ? 'border-blue-600 text-blue-600'
                        : 'border-transparent text-gray-600 hover:text-gray-900'}"
                >
                    Dashboard
                </a>
                <a
                    href="/mocks"
                    class="px-6 py-3 font-medium border-b-2 {$page.url.pathname.startsWith(
                        '/mocks',
                    )
                        ? 'border-blue-600 text-blue-600'
                        : 'border-transparent text-gray-600 hover:text-gray-900'}"
                >
                    Mocks
                </a>
                <a
                    href="/modifiers"
                    class="px-6 py-3 font-medium border-b-2 {$page.url.pathname.startsWith(
                        '/modifiers',
                    )
                        ? 'border-blue-600 text-blue-600'
                        : 'border-transparent text-gray-600 hover:text-gray-900'}"
                >
                    Modifiers
                </a>

                <a
                    href="/rate-limits"
                    class="px-6 py-3 font-medium border-b-2 {$page.url.pathname.startsWith(
                        '/rate-limits',
                    )
                        ? 'border-blue-600 text-blue-600'
                        : 'border-transparent text-gray-600 hover:text-gray-900'}"
                >
                    Rate Limits
                </a>
                <a
                    href="/latency"
                    class="px-6 py-3 font-medium border-b-2 {$page.url.pathname.startsWith(
                        '/latency',
                    )
                        ? 'border-blue-600 text-blue-600'
                        : 'border-transparent text-gray-600 hover:text-gray-900'}"
                >
                    Latency Injection
                </a>
            </div>
        </div>
    </nav>

    <main class="flex-1">
        <slot />
    </main>
</div>
