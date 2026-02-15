<script lang="ts">
	import { page } from "$app/state";
	import { Video, Film } from "lucide-svelte";
	import "../app.css";

	let { children } = $props();
</script>

<svelte:head>
	<meta charset="UTF-8" />
	<meta name="viewport" content="width=device-width, initial-scale=1.0" />
</svelte:head>

<div class="navbar bg-base-100 shadow-sm mb-5 rounded-box">
	<div class="flex-1">
		<a class="btn btn-ghost text-xl" href="/">️WebCam</a>
	</div>
	<div class="flex-none">
		<ul class="menu menu-horizontal px-1">
			<li>
				<a
					href="/"
					class={page.url.pathname === "/"
						? "bg-neutral text-neutral-content"
						: ""}
				>
					<Video size="20" />
					ライブ
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
					録画
				</a>
			</li>
		</ul>
	</div>
</div>

<div class="container mx-auto px-4">
	{@render children()}
</div>
