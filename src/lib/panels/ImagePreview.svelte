<script lang="ts">
  // Previsualización de imagen en pestaña de solo-lectura (PLAN.md §6.1).
  // Lee los bytes via el comando `read_asset_bytes` (ver src-tauri/src/assets.rs)
  // y arma un Blob local — evita abrir el protocolo `asset:`/su scope de
  // filesystem para carpetas de sketch arbitrarias, ver nota en assets.rs.
  import { invoke } from "@tauri-apps/api/core";
  import { t } from "$lib/i18n";

  let { projectPath, filename }: { projectPath: string; filename: string } = $props();

  let url = $state<string | null>(null);
  let error = $state<string | null>(null);
  let dims = $state<{ w: number; h: number } | null>(null);

  const MIME: Record<string, string> = {
    png: "image/png",
    gif: "image/gif",
    jpg: "image/jpeg",
    jpeg: "image/jpeg",
  };

  function mimeFor(name: string): string {
    const ext = name.split(".").pop()?.toLowerCase() ?? "";
    return MIME[ext] ?? "application/octet-stream";
  }

  async function load(name: string) {
    error = null;
    dims = null;
    const previous = url;
    url = null;
    try {
      const bytes = await invoke<number[]>("read_asset_bytes", {
        projectPath,
        kind: "images",
        filename: name,
      });
      const blob = new Blob([new Uint8Array(bytes)], { type: mimeFor(name) });
      url = URL.createObjectURL(blob);
    } catch (e) {
      error = `${t("preview.loadError")}: ${e}`;
    }
    if (previous) URL.revokeObjectURL(previous);
  }

  function handleLoad(e: Event) {
    const img = e.currentTarget as HTMLImageElement;
    dims = { w: img.naturalWidth, h: img.naturalHeight };
  }

  // ponytail: si el navegador bloquea/falla la carga del blob (CSP, archivo
  // corrupto, etc.) esto evita quedarnos en blanco sin explicar nada.
  function handleImgError() {
    error = `${t("preview.loadError")}: no se pudo mostrar la imagen`;
  }

  $effect(() => {
    load(filename);
  });

  $effect(() => {
    return () => {
      if (url) URL.revokeObjectURL(url);
    };
  });
</script>

<div class="image-preview">
  {#if error}
    <p class="error">{error}</p>
  {:else if url}
    <div class="canvas">
      <img src={url} alt={filename} onload={handleLoad} onerror={handleImgError} />
    </div>
    <p class="meta">
      {filename}
      {#if dims}
        · {dims.w}×{dims.h} {t("preview.dimensions")}
      {/if}
    </p>
  {/if}
</div>

<style>
  .image-preview {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--az-space-2);
    padding: var(--az-space-3);
    overflow: auto;
  }
  .canvas {
    display: flex;
    align-items: center;
    justify-content: center;
    max-width: 90%;
    max-height: 80%;
    padding: var(--az-space-3);
    background-image:
      linear-gradient(45deg, #808080 25%, transparent 25%),
      linear-gradient(-45deg, #808080 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, #808080 75%),
      linear-gradient(-45deg, transparent 75%, #808080 75%);
    background-size: 16px 16px;
    background-position: 0 0, 0 8px, 8px -8px, -8px 0px;
    background-color: #b0b0b0;
    border: 1px solid var(--az-color-border);
    border-radius: var(--az-radius);
  }
  img {
    max-width: 100%;
    max-height: 60vh;
    image-rendering: pixelated;
  }
  .meta {
    color: var(--az-color-text-muted);
    font-size: 0.85rem;
  }
  .error {
    color: var(--az-color-danger);
  }
</style>
