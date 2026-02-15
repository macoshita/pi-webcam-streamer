<script lang="ts">
  import { settings } from "$lib/stores/settings.svelte";
</script>

<svelte:head>
  <title>{settings.t.live}</title>
</svelte:head>

<div class="card bg-base-100 shadow-xl overflow-hidden">
  <div class="card-body p-0 overflow-hidden">
    <div class="relative w-full text-center bg-black">
      <img
        src="/api/stream"
        alt="Webcam Stream"
        class="w-full h-auto object-contain max-h-[70vh] mx-auto"
      />
      <div class="absolute top-2 left-2">
        <div class="badge badge-error gap-2 text-white animate-pulse">LIVE</div>
      </div>
    </div>
  </div>
</div>
