<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import Hls from "hls.js";

  let videoElement: HTMLVideoElement;
  let hls: Hls | undefined;

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

<video bind:this={videoElement} controls autoplay>
  <track kind="captions" />
</video>

<style>
  video {
    max-width: 100%;
    height: auto;
    border: 2px solid #333;
    border-radius: 8px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    background-color: #000;
  }
</style>
