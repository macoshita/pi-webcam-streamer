<script lang="ts">
	import { page } from "$app/state";
	import { browser } from "$app/environment";
	import { Video, Film, Settings as SettingsIcon, Check } from "lucide-svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import "../app.css";

	let { children } = $props();

	$effect(() => {
		if (!browser) return;

		localStorage.setItem("language", settings.language);
		localStorage.setItem("theme", settings.theme);

		const updateTheme = () => {
			let theme = settings.theme;
			if (theme === "system") {
				theme = window.matchMedia("(prefers-color-scheme: dark)").matches
					? "dark"
					: "light";
			}
			document.documentElement.setAttribute("data-theme", theme);
		};

		updateTheme();

		const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
		const handleChange = () => {
			if (settings.theme === "system") updateTheme();
		};
		mediaQuery.addEventListener("change", handleChange);
		return () => mediaQuery.removeEventListener("change", handleChange);
	});
</script>

<svelte:head>
	<meta charset="UTF-8" />
	<meta name="viewport" content="width=device-width, initial-scale=1.0" />
</svelte:head>

<div class="navbar bg-base-100 shadow-sm mb-5 rounded-box">
	<div class="flex-1">
		<ul class="menu menu-horizontal px-1">
			<li>
				<a
					href="/"
					class={page.url.pathname === "/"
						? "bg-neutral text-neutral-content"
						: ""}
				>
					<Video size="20" />
					{settings.t.live}
				</a>
			</li>
			<li>
				<a
					href="/recordings"
					class={page.url.pathname === "/recordings"
						? "bg-neutral text-neutral-content"
						: ""}
				>
					<Film size="20" />
					{settings.t.recordings}
				</a>
			</li>
		</ul>
	</div>
	<div class="flex-none">
		<ul class="menu menu-horizontal px-1">
			<li>
				<details>
					<summary>
						<SettingsIcon size="20" />
					</summary>
					<ul
						class="bg-base-100 rounded-t-none p-2 z-[1] shadow-sm min-w-40 right-0"
					>
						<li class="menu-title">{settings.t.language}</li>
						<li>
							<button
								class={settings.language === "en" ? "active" : ""}
								onclick={() => (settings.language = "en")}
							>
								{#if settings.language === "en"}
									<Check size="16" />
								{/if}
								{settings.t.english}
							</button>
						</li>
						<li>
							<button
								class={settings.language === "ja" ? "active" : ""}
								onclick={() => (settings.language = "ja")}
							>
								{#if settings.language === "ja"}
									<Check size="16" />
								{/if}
								{settings.t.japanese}
							</button>
						</li>
						<li class="menu-title mt-2">{settings.t.theme}</li>
						<li>
							<button
								class={settings.theme === "light" ? "active" : ""}
								onclick={() => (settings.theme = "light")}
							>
								{#if settings.theme === "light"}
									<Check size="16" />
								{/if}
								{settings.t.light}
							</button>
						</li>
						<li>
							<button
								class={settings.theme === "dark" ? "active" : ""}
								onclick={() => (settings.theme = "dark")}
							>
								{#if settings.theme === "dark"}
									<Check size="16" />
								{/if}
								{settings.t.dark}
							</button>
						</li>
						<li>
							<button
								class={settings.theme === "system" ? "active" : ""}
								onclick={() => (settings.theme = "system")}
							>
								{#if settings.theme === "system"}
									<Check size="16" />
								{/if}
								{settings.t.system}
							</button>
						</li>
					</ul>
				</details>
			</li>
		</ul>
	</div>
</div>

<div class="container mx-auto px-4">
	{@render children()}
</div>
