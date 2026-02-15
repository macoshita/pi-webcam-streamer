<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import Hls from "hls.js";
  import { Play, Pause } from "lucide-svelte";

  let videoElement: HTMLVideoElement;
  let seekBar: HTMLInputElement;
  let hls: Hls | undefined;
  let currentDateTime = "";
  let isPlaying = false;
  let duration = 0;
  let currentTime = 0;
  let isSeeking = false;

  // セグメントの実際の開始日時（Dateオブジェクト）とHLSタイムライン上の開始時間
  // FRAG_CHANGED で更新されるので、配列として全セグメント情報を保持
  type SegmentInfo = { hlsStart: number; date: Date };
  let segments: SegmentInfo[] = [];

  // ライブエッジからのマージン（秒）。この分だけシーク可能範囲を縮める
  const LIVE_EDGE_MARGIN = 30;

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

  /** 最大シーク可能時間: duration から LIVE_EDGE_MARGIN を引いた値 */
  function safeMaxTime(): number {
    if (!duration || !isFinite(duration)) return 0;
    return Math.max(0, duration - LIVE_EDGE_MARGIN);
  }

  /** HLSタイムライン上の時刻から実際の撮影日時へ変換 */
  function hlsTimeToDate(t: number): Date | null {
    if (segments.length === 0) return null;
    // tに最も近い（それ以前の）セグメントを探す
    let best: SegmentInfo | null = null;
    for (const seg of segments) {
      if (seg.hlsStart <= t) {
        best = seg;
      } else {
        break;
      }
    }
    if (!best) best = segments[0];
    const offset = t - best.hlsStart;
    return new Date(best.date.getTime() + offset * 1000);
  }

  function handleTimeUpdate() {
    if (!videoElement) return;
    currentTime = videoElement.currentTime;
    isPlaying = !videoElement.paused;

    const date = hlsTimeToDate(currentTime);
    if (date) {
      currentDateTime = formatDateTime(date);
    }

    // シークバー操作中でなければ値を同期（DOM直接操作）
    if (!isSeeking && seekBar) {
      seekBar.max = String(safeMaxTime());
      seekBar.value = String(currentTime);
    }
  }

  function handleDurationChange() {
    if (videoElement) {
      duration = videoElement.duration || 0;
    }
  }

  function togglePlayPause() {
    if (!videoElement) return;
    if (videoElement.paused) {
      videoElement.play().catch(() => {});
    } else {
      videoElement.pause();
    }
    isPlaying = !videoElement.paused;
  }

  function onSeekStart() {
    isSeeking = true;
  }

  function onSeekInput(e: Event) {
    const target = e.target as HTMLInputElement;
    const val = parseFloat(target.value);
    // ライブエッジを越えないようにクランプ
    const clamped = Math.min(val, safeMaxTime());
    target.value = String(clamped);

    // ドラッグ中も映像を更新
    if (videoElement) {
      videoElement.currentTime = clamped;
    }

    // 日時表示を更新
    const date = hlsTimeToDate(clamped);
    if (date) {
      currentDateTime = formatDateTime(date);
    }
  }

  function onSeekEnd() {
    if (!videoElement || !seekBar) return;
    const clamped = Math.min(parseFloat(seekBar.value), safeMaxTime());
    videoElement.currentTime = clamped;
    isSeeking = false;
  }

  /** シークバーの現在位置に対応する撮影日時テキスト */
  function seekBarDateLabel(t: number): string {
    const date = hlsTimeToDate(t);
    return date ? formatDateTime(date) : "";
  }

  onMount(() => {
    const videoSrc = "/api/videos/playlist.m3u8";

    if (Hls.isSupported()) {
      hls = new Hls({
        debug: false,
        enableWorker: true,
        lowLatencyMode: false,
        backBufferLength: 90,
      });
      hls.loadSource(videoSrc);
      hls.attachMedia(videoElement);
      hls.on(Hls.Events.MANIFEST_PARSED, () => {
        videoElement.play().catch((e: Error) => {
          console.log("Auto-play prevented: " + e);
        });
      });
      hls.on(Hls.Events.LEVEL_LOADED, (_event: string, data: any) => {
        const details = data.details;
        if (!details || !details.fragments) return;
        // 全フラグメント情報からセグメントマップを再構築
        const newSegments: SegmentInfo[] = [];
        for (const frag of details.fragments) {
          const fragUrl = frag.relurl || frag.url || "";
          const date = parseSegmentDate(fragUrl);
          if (date) {
            newSegments.push({ hlsStart: frag.start ?? 0, date });
          }
        }
        if (newSegments.length > 0) {
          segments = newSegments;
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
  <title>録画再生</title>
</svelte:head>

<div class="card bg-base-100 shadow-xl overflow-hidden">
  <div class="relative bg-black">
    {#if currentDateTime}
      <div class="absolute top-4 left-4 z-10">
        <div class="badge badge-neutral gap-2 font-mono opacity-80">
          {currentDateTime}
        </div>
      </div>
    {/if}
    <!-- svelte-ignore a11y_media_has_caption -->
    <video
      bind:this={videoElement}
      autoplay
      on:timeupdate={handleTimeUpdate}
      on:durationchange={handleDurationChange}
      on:play={() => (isPlaying = true)}
      on:pause={() => (isPlaying = false)}
      class="w-full h-auto max-h-[70vh] object-contain mx-auto"
    ></video>
  </div>

  <div class="p-4 bg-base-200 flex items-center gap-4">
    <button class="btn btn-circle btn-primary" on:click={togglePlayPause}>
      {#if isPlaying}
        <Pause class="h-6 w-6" />
      {:else}
        <Play class="h-6 w-6" />
      {/if}
    </button>

    <div class="flex-1">
      <input
        bind:this={seekBar}
        type="range"
        min="0"
        max={safeMaxTime()}
        step="0.1"
        class="range range-xs range-primary w-full"
        on:mousedown={onSeekStart}
        on:touchstart={onSeekStart}
        on:input={onSeekInput}
        on:mouseup={onSeekEnd}
        on:touchend={onSeekEnd}
      />
    </div>
  </div>
</div>
