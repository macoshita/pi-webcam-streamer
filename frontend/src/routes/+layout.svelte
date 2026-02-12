<script lang="ts">
	import { page } from "$app/state";

	let { children } = $props();
</script>

<svelte:head>
	<meta charset="UTF-8" />
	<meta name="viewport" content="width=device-width, initial-scale=1.0" />
</svelte:head>

<nav class="nav">
	<a href="/" class:active={page.url.pathname === "/"}>ライブ映像</a>
	<a href="/recordings" class:active={page.url.pathname === "/recordings"}
		>録画再生</a
	>
</nav>

{@render children()}

<style>
	:global(body) {
		font-family: Arial, sans-serif;
		max-width: 800px;
		margin: 20px auto;
		padding: 0 10px;
		text-align: center;
		background-color: #f4f4f9;
	}

	.nav {
		margin-bottom: 20px;
		padding: 10px;
		background: #fff;
		border-radius: 8px;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
		display: inline-block;
	}

	.nav a {
		margin: 0 10px;
		color: #007bff;
		text-decoration: none;
		font-weight: bold;
		font-size: 1.1rem;
		padding: 10px;
	}

	.nav a:hover {
		text-decoration: underline;
		color: #0056b3;
	}

	.nav a.active {
		color: #333;
		pointer-events: none;
		text-decoration: none;
		border-bottom: 2px solid #333;
	}
</style>
