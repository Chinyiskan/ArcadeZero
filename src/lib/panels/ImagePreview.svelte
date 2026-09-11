<script lang="ts">
  // Previsualización de imagen en pestaña de solo-lectura (PLAN.md §6.1).
  // Lee los bytes via el comando `read_asset_bytes` (ver src-tauri/src/assets.rs)
  // y arma un data: URI local — evita abrir el protocolo `asset:`/su scope de
  // filesystem para carpetas de sketch arbitrarias (ver nota en assets.rs), y
  // evita Blob/createObjectURL (dio problemas de carga/revoke en el webview).
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

  // ponytail: data: URI en vez de Blob/createObjectURL — mas simple (sin
  // revoke que gestionar) y la CSP ya permite `data:` sin depender del
  // soporte de `blob:` del webview. En chunks de 8KB para no reventar el
  // limite de argumentos de String.fromCharCode con imagenes grandes.
  function bytesToBase64(bytes: Uint8Array): string {
    let binary = "";
    const chunkSize = 8192;
    for (let i = 0; i < bytes.length; i += chunkSize) {
      binary += String.fromCharCode(...bytes.subarray(i, i + chunkSize));
    }
    return btoa(binary);
  }

  async function load(name: string) {
    error = null;
    dims = null;
    url = null;
    try {
      const bytes = await invoke<number[]>("read_asset_bytes", {
        projectPath,
        kind: "images",
        filename: name,
      });
      const base64 = bytesToBase64(new Uint8Array(bytes));
      url = `data:${mimeFor(name)};base64,${base64}`;
    } catch (e) {
      error = `${t("preview.loadError")}: ${e}`;
    }
  }

  function handleLoad(e: Event) {
    const img = e.currentTarget as HTMLImageElement;
    dims = { w: img.naturalWidth, h: img.naturalHeight };
  }

  // ponytail: si la imagen no decodifica (archivo corrupto, etc.) esto evita
  // quedarnos en blanco sin explicar nada.
  function handleImgError() {
    error = `${t("preview.loadError")}: no se pudo mostrar la imagen`;
  }

  $effect(() => {
    load(filename);
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
