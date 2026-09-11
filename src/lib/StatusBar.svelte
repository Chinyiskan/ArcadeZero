<script lang="ts">
  import { t } from "$lib/i18n";
  import CheckCircleIcon from "phosphor-svelte/lib/CheckCircleIcon";
  import WarningCircleIcon from "phosphor-svelte/lib/WarningCircleIcon";

  let {
    fileName = "main.py",
    line = 1,
    col = 1,
    saved = true,
    runtimeOk = null,
  }: {
    fileName?: string;
    line?: number;
    col?: number;
    saved?: boolean;
    runtimeOk?: boolean | null;
  } = $props();
</script>

<div class="statusbar">
  <span>{fileName}</span>
  <span>· línea {line}, col {col}</span>
  <span class="badge">
    · {saved ? t("status.saved") : t("status.unsaved")}
    {#if saved}
      <CheckCircleIcon size={14} weight="fill" aria-hidden="true" />
    {/if}
  </span>
  {#if runtimeOk !== null}
    <span class="badge">
      · Python 3.13
      {#if runtimeOk}
        <CheckCircleIcon size={14} weight="fill" aria-hidden="true" />
      {:else}
        <WarningCircleIcon size={14} weight="fill" aria-hidden="true" />
      {/if}
    </span>
  {/if}
</div>

<style>
  .statusbar {
    display: flex;
    gap: var(--az-space-2);
    padding: var(--az-space-1) var(--az-space-3);
    font-size: 0.8rem;
    color: var(--az-color-text-muted);
    background: var(--az-color-panel-bg);
    border-top: 1px solid var(--az-color-border);
  }
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 2px;
  }
</style>
