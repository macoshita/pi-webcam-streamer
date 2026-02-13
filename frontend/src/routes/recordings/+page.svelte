<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import Hls from "hls.js";

  let videoElement: HTMLVideoElement;
  let hls: Hls | undefined;
  let currentDateTime = "";

  // セグメントの実際の開始日時（Dateオブジェクト）とHLSタイムライン上の開始時間
  let segmentStartDate: Date | null = null;
  let segmentHlsStart = 0;

  function parseSegmentDate(url: string): Date | null {
    const match = url.match(/(\d{4})(\d{2})(\d{2})_(\d{2})(\d{2})(\d{2})\.mp4/);
    if (!match) return null;
    const [, year, month, day, hour, min, sec] = match;
    return new Date(+year, +month - 1, +day, +hour, +min, +sec);
  }

  function formatDateTime(date: Date): string {
    const pad = (n: number) => n.toString().padStart(2, "0");
    return `${date.getFullYear()}/${pad(date.getMonth() + 1)}/${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
  }

  function handleTimeUpdate() {
    if (!segmentStartDate || !videoElement) return;
    const offsetSec = videoElement.currentTime - segmentHlsStart;
    const displayDate = new Date(segmentStartDate.getTime() + offsetSec * 1000);
    currentDateTime = formatDateTime(displayDate);
  }

  onMount(() => {
    const videoSrc = "/api/videos/playlist.m3u8";

    if (Hls.isSupported()) {
      hls = new Hls({
        debug: false,
        enableWorker: true,
        lowLatencyMode: true,
        backBufferLength: 90,
      });
      hls.loadSource(videoSrc);
      hls.attachMedia(videoElement);
      hls.on(Hls.Events.MANIFEST_PARSED, () => {
        videoElement.play().catch((e: Error) => {
          console.log("Auto-play prevented: " + e);
        });
      });
      hls.on(Hls.Events.FRAG_CHANGED, (_event: string, data: any) => {
        const frag = data.frag;
        if (!frag) return;
        const fragUrl = frag.relurl || frag.url || "";
        const date = parseSegmentDate(fragUrl);
        if (date) {
          segmentStartDate = date;
          segmentHlsStart = frag.start ?? 0;
        }
      });
      hls.on(Hls.Events.ERROR, (_event: string, data: any) => {
        if (data.fatal) {
          switch (data.type) {
            case Hls.ErrorTypes.NETWORK_ERROR:
              console.log("fatal network error encountered, try to recover");
              hls?.startLoad();
              break;
            case Hls.ErrorTypes.MEDIA_ERROR:
              console.log("fatal media error encountered, try to recover");
              hls?.recoverMediaError();
              break;
            default:
              hls?.destroy();
              break;
          }
        }
      });
    } else if (videoElement.canPlayType("application/vnd.apple.mpegurl")) {
      videoElement.src = videoSrc;
      videoElement.addEventListener("loadedmetadata", () => {
        videoElement.play();
      });
    }
  });

  onDestroy(() => {
    if (hls) {
      hls.destroy();
    }
  });
</script>

<svelte:head>
  <title>Video Player</title>
</svelte:head>

<div class="player-container">
  {#if currentDateTime}
    <div class="datetime-overlay">{currentDateTime}</div>
  {/if}
  <video
    bind:this={videoElement}
    controls
    autoplay
    on:timeupdate={handleTimeUpdate}
  >
    <track kind="captions" />
  </video>
</div>

<style>
  .player-container {
    position: relative;
    display: inline-block;
    max-width: 100%;
  }

  .datetime-overlay {
    position: absolute;
    top: 8px;
    left: 8px;
    background: rgba(0, 0, 0, 0.7);
    color: #fff;
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 14px;
    font-family: monospace;
    z-index: 10;
    pointer-events: none;
  }

  video {
    max-width: 100%;
    height: auto;
    border: 2px solid #333;
    border-radius: 8px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    background-color: #000;
  }
</style>
