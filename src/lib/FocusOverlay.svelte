<script lang="ts">
  import { untrack } from 'svelte';
  import { clip } from '../stores/clip.svelte';
  import { playerRef } from './playerRef.svelte';
  import { videoDisplayRect, clampRegion, sourceToCss, defaultRegion } from './focus/coords';
  import { interpolateRegion } from './focus/interpolate';
  import { aspectRatioValue, type Region } from './focus/types';

  let svgEl: SVGSVGElement | null = $state(null);
  type DragMode = 'move' | 'nw' | 'ne' | 'sw' | 'se' | 'n' | 's' | 'e' | 'w' | null;
  let dragMode: DragMode = $state(null);
  let dragStart = { x: 0, y: 0 };
  let regionAtDragStart: Region = { x: 0, y: 0, width: 0, height: 0 };
  let lastSampleAt = 0;
  let currentTime = $state(0);

  const probe = $derived(clip.probe);
  const ft = $derived(clip.focusTools);

  function sourceSize() {
    return probe ? { width: probe.width, height: probe.height } : { width: 0, height: 0 };
  }

  function lockAspect(): number | null {
    if (ft.mode === 'reframe' && probe) {
      return aspectRatioValue(ft.aspectRatio, { width: probe.width, height: probe.height });
    }
    if (ft.shape === 'circle') return 1;
    return null;
  }

  $effect(() => {
    if (!probe) return;
    const src = sourceSize();
    const r = ft.region;
    const tooSmall = r.width < 2 || r.height < 2;
    const offScreen = r.x + r.width <= 0 || r.y + r.height <= 0
                    || r.x >= src.width || r.y >= src.height;
    if (tooSmall || offScreen) {
      ft.region = defaultRegion(src, lockAspect());
    }
  });

  $effect(() => {
    const v = playerRef.video;
    if (!v) return;
    const onTime = () => { currentTime = v.currentTime; };
    v.addEventListener('timeupdate', onTime);
    v.addEventListener('seeked', onTime);
    return () => {
      v.removeEventListener('timeupdate', onTime);
      v.removeEventListener('seeked', onTime);
    };
  });

  $effect(() => {
    // Only depend on currentTime — drag state and recording flips shouldn't trigger re-interp.
    void currentTime;
    if (untrack(() => dragMode)) return;
    if (untrack(() => ft.recording.active)) return;
    const kfs = untrack(() => ft.keyframes);
    const sel = untrack(() => ft.selectedKeyframeIndex);
    if (kfs.length > 0 && sel === null) {
      const interp = interpolateRegion(kfs, currentTime);
      if (interp) ft.region = interp;
    }
  });

  let _tick = $state(0);
  $effect(() => {
    const onResize = () => { _tick++; };
    window.addEventListener('resize', onResize);
    window.addEventListener('scroll', onResize, true);
    return () => {
      window.removeEventListener('resize', onResize);
      window.removeEventListener('scroll', onResize, true);
    };
  });

  const display = $derived.by(() => {
    void _tick;
    const v = playerRef.video;
    if (!v || !probe) return null;
    return videoDisplayRect(v, { width: probe.width, height: probe.height });
  });

  const cssRegion = $derived.by(() => {
    if (!display || !probe) return null;
    return sourceToCss(ft.region, display, { width: probe.width, height: probe.height });
  });

  let smoothedRegion: Region | null = null;
  function recordSample() {
    if (!ft.recording.active) return;
    const v = playerRef.video;
    if (!v) return;
    const now = performance.now();
    if (now - lastSampleAt < ft.recording.sampleIntervalMs) return;
    lastSampleAt = now;
    // Exponential moving average on the live region for jitter reduction.
    // smoothing in [0..1]: 0 = raw, 1 = max smoothing (very laggy). Map to alpha.
    const s = Math.max(0, Math.min(1, ft.recording.smoothing));
    const alpha = 1 - s * 0.85; // alpha in [0.15..1]; 1 = use raw, lower = trust history more
    if (!smoothedRegion) {
      smoothedRegion = { ...ft.region };
    } else {
      smoothedRegion = {
        x: alpha * ft.region.x + (1 - alpha) * smoothedRegion.x,
        y: alpha * ft.region.y + (1 - alpha) * smoothedRegion.y,
        width: alpha * ft.region.width + (1 - alpha) * smoothedRegion.width,
        height: alpha * ft.region.height + (1 - alpha) * smoothedRegion.height,
      };
    }
    clip.upsertKeyframeAtTime(v.currentTime, smoothedRegion, ft.recording.sampleIntervalMs / 1000 / 2);
  }

  $effect(() => {
    if (!ft.recording.active) smoothedRegion = null;
  });

  function startDrag(e: PointerEvent, mode: typeof dragMode) {
    if (!display || !probe) return;
    e.stopPropagation();
    e.preventDefault();
    dragMode = mode;
    dragStart = { x: e.clientX, y: e.clientY };
    regionAtDragStart = { ...ft.region };
    ft.interactionHeld = true;
    (e.target as Element).setPointerCapture?.(e.pointerId);
    // Keep keyboard focus on body so the R hotkey continues to work while dragging.
    if (document.activeElement instanceof HTMLElement) {
      document.activeElement.blur();
    }
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragMode || !display || !probe) return;
    const sx = probe.width / display.width;
    const sy = probe.height / display.height;
    const dxSrc = (e.clientX - dragStart.x) * sx;
    const dySrc = (e.clientY - dragStart.y) * sy;

    const r = { ...regionAtDragStart };
    if (dragMode === 'move') {
      r.x += dxSrc;
      r.y += dySrc;
    } else {
      if (dragMode.includes('e')) r.width = regionAtDragStart.width + dxSrc;
      if (dragMode.includes('w')) {
        r.x = regionAtDragStart.x + dxSrc;
        r.width = regionAtDragStart.width - dxSrc;
      }
      if (dragMode.includes('s')) r.height = regionAtDragStart.height + dySrc;
      if (dragMode.includes('n')) {
        r.y = regionAtDragStart.y + dySrc;
        r.height = regionAtDragStart.height - dySrc;
      }
    }
    ft.region = clampRegion(r, sourceSize(), { lockAspect: lockAspect() });

    if (ft.selectedKeyframeIndex !== null) {
      const idx = ft.selectedKeyframeIndex;
      const k = ft.keyframes[idx];
      ft.keyframes[idx] = { time: k.time, ...ft.region };
    }
    recordSample();
  }

  function onPointerUp(e: PointerEvent) {
    dragMode = null;
    if (ft.interactionHeld) {
      ft.interactionHeld = false;
      // If we started a box-hold + R recording session, releasing the box ends recording too.
      if (ft.recording.holdMode && ft.recording.active) {
        ft.recording.active = false;
        ft.recording.holdMode = false;
        clip.sortKeyframes();
      }
    }
    (e.target as Element).releasePointerCapture?.(e.pointerId);
  }

  $effect(() => {
    if (!ft.recording.active) return;
    let raf = 0;
    const tick = () => {
      recordSample();
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  });

  function strokeColor() { return ft.style.strokeColor || '#ff3b30'; }
  function fillColor() { return ft.style.fillColor || '#ff3b30'; }
</script>

<svelte:window onpointermove={onPointerMove} onpointerup={onPointerUp} />

{#if display && cssRegion && ft.enabled && ft.mode !== 'off'}
  {@const rx = cssRegion.x - display.left}
  {@const ry = cssRegion.y - display.top}
  {@const rw = cssRegion.width}
  {@const rh = cssRegion.height}
  {@const cx = rx + rw / 2}
  {@const cy = ry + rh / 2}
  {@const isCircle = ft.shape === 'circle'}
  {@const sw = Math.max(1, ft.style.strokeWidth)}
  <svg
    bind:this={svgEl}
    class="focus-overlay"
    style="position: fixed; left: {display.left}px; top: {display.top}px; width: {display.width}px; height: {display.height}px; pointer-events: none; z-index: 5;"
  >
    {#if ft.mode === 'spotlight'}
      <defs>
        <mask id="focus-spot-mask">
          <rect x="0" y="0" width={display.width} height={display.height} fill="white" />
          {#if isCircle}
            <ellipse cx={cx} cy={cy} rx={rw / 2} ry={rh / 2} fill="black" />
          {:else}
            <rect x={rx} y={ry} width={rw} height={rh} fill="black" />
          {/if}
        </mask>
      </defs>
      <rect
        x="0" y="0"
        width={display.width}
        height={display.height}
        fill="black"
        fill-opacity={ft.style.spotlightDim}
        mask="url(#focus-spot-mask)"
        style="pointer-events: none;"
      />
    {/if}

    {#if ft.style.fillOpacity > 0 && (ft.mode === 'box' || ft.mode === 'label' || ft.mode === 'reframe')}
      {#if isCircle}
        <ellipse
          cx={cx} cy={cy} rx={rw / 2} ry={rh / 2}
          fill={fillColor()}
          fill-opacity={ft.style.fillOpacity}
          style="pointer-events: none;"
        />
      {:else}
        <rect
          x={rx} y={ry} width={rw} height={rh}
          fill={fillColor()}
          fill-opacity={ft.style.fillOpacity}
          style="pointer-events: none;"
        />
      {/if}
    {/if}

    <!-- Halo outline (dark) for contrast against any background. -->
    {#if isCircle}
      <ellipse
        cx={cx} cy={cy} rx={rw / 2} ry={rh / 2}
        fill="transparent"
        stroke="rgba(0,0,0,0.55)"
        stroke-width={sw + 2}
        style="pointer-events: none;"
      />
    {:else}
      <rect
        x={rx} y={ry} width={rw} height={rh}
        fill="transparent"
        stroke="rgba(0,0,0,0.55)"
        stroke-width={sw + 2}
        style="pointer-events: none;"
      />
    {/if}

    <!-- Visible stroke -->
    {#if isCircle}
      <ellipse
        cx={cx} cy={cy} rx={rw / 2} ry={rh / 2}
        fill="transparent"
        stroke={strokeColor()}
        stroke-width={sw}
        style="pointer-events: none;"
      />
    {:else}
      <rect
        x={rx} y={ry} width={rw} height={rh}
        fill="transparent"
        stroke={strokeColor()}
        stroke-width={sw}
        style="pointer-events: none;"
      />
    {/if}

    <!-- Invisible drag-target (wider than the visible stroke for easy grabbing).
         Deliberately not focusable — keeps document focus on body so R hotkey reaches the window. -->
    {#if isCircle}
      <ellipse
        role="presentation"
        cx={cx} cy={cy} rx={rw / 2} ry={rh / 2}
        fill="transparent"
        stroke="transparent"
        stroke-width="14"
        pointer-events="stroke"
        style="cursor: move;"
        onpointerdown={(e) => startDrag(e, 'move')}
      />
      <ellipse
        role="presentation"
        cx={cx} cy={cy} rx={Math.max(0, rw / 2 - 8)} ry={Math.max(0, rh / 2 - 8)}
        fill="rgba(0,0,0,0.001)"
        pointer-events="fill"
        style="cursor: move;"
        onpointerdown={(e) => startDrag(e, 'move')}
      />
    {:else}
      <rect
        role="presentation"
        x={rx} y={ry} width={rw} height={rh}
        fill="rgba(0,0,0,0.001)"
        stroke="transparent"
        stroke-width="14"
        pointer-events="all"
        style="cursor: move;"
        onpointerdown={(e) => startDrag(e, 'move')}
      />
    {/if}

    {#if ft.mode === 'label' && ft.label.text}
      {@const tx = ft.label.position === 'left' ? rx - 8
                  : ft.label.position === 'right' ? rx + rw + 8
                  : rx + rw / 2}
      {@const ty = ft.label.position === 'top' ? ry - 8
                  : ft.label.position === 'bottom' ? ry + rh + 18
                  : ry + rh / 2}
      {@const anchor = ft.label.position === 'left' ? 'end'
                      : ft.label.position === 'right' ? 'start'
                      : 'middle'}
      <text
        x={tx} y={ty}
        text-anchor={anchor}
        fill={ft.label.textColor}
        style="paint-order: stroke; stroke: {ft.label.backgroundColor}; stroke-width: 4px; font: 600 13px system-ui; pointer-events: none;"
      >{ft.label.text}</text>
    {/if}

    {#each ['nw','n','ne','e','se','s','sw','w'] as h}
      {@const hx = rx + (h.includes('w') ? 0 : h.includes('e') ? rw : rw / 2)}
      {@const hy = ry + (h.includes('n') ? 0 : h.includes('s') ? rh : rh / 2)}
      {@const cursor = h === 'n' || h === 's' ? 'ns-resize'
                      : h === 'e' || h === 'w' ? 'ew-resize'
                      : h === 'nw' || h === 'se' ? 'nwse-resize'
                      : 'nesw-resize'}
      <!-- Larger transparent hit target around the dot -->
      <circle
        role="presentation"
        cx={hx} cy={hy} r="10"
        fill="rgba(0,0,0,0.001)"
        style="pointer-events: all; cursor: {cursor};"
        onpointerdown={(e) => startDrag(e, h as any)}
      />
      <!-- Visible dot -->
      <circle
        role="presentation"
        cx={hx} cy={hy} r="4"
        fill="white"
        stroke={strokeColor()}
        stroke-width="1.5"
        style="pointer-events: none; filter: drop-shadow(0 0 2px rgba(0,0,0,0.6));"
      />
    {/each}
  </svg>
{/if}
