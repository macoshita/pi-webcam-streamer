<svelte:head>
  <title>Webcam Stream</title>
</svelte:head>

<div class="card bg-base-100 shadow-xl overflow-hidden">
  <div class="card-body p-0 overflow-hidden">
    <div class="relative w-full text-center bg-black">
      <div class="absolute top-4 left-4 z-10">
        <div class="badge badge-error gap-2 text-white animate-pulse">LIVE</div>
      </div>
      <img
        src="/api/stream"
        alt="Webcam Stream"
        class="w-full h-auto object-contain max-h-[70vh] mx-auto"
      />
    </div>
  </div>
</div>
