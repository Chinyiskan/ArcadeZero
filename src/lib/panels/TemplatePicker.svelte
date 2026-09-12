<script lang="ts">
  // Selector de plantillas (PLAN.md §5/§11 Fase 4): reemplaza el
  // "en-blanco" hardcodeado de handleNew() en +page.svelte. Se usa tanto
  // desde Nuevo (toolbar/onboarding) como desde Ayuda ("Ver plantillas").
  import { t } from "$lib/i18n";
  import { TEMPLATE_IDS, RECOMMENDED_TEMPLATE, type TemplateId } from "$lib/templates";
  import { focusTrap } from "$lib/actions/focusTrap";

  let {
    open = false,
    onpick,
    onclose,
  }: {
    open?: boolean;
    onpick?: (id: TemplateId) => void;
    onclose?: () => void;
  } = $props();
</script>

{#if open}
  <div
    class="backdrop"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) onclose?.();
    }}
  >
    <div
      class="modal"
      role="dialog"
      aria-modal="true"
      aria-label={t("templates.pickTitle")}
      tabindex="-1"
      use:focusTrap
    >
      <h2>{t("templates.pickTitle")}</h2>
      <p class="body">{t("templates.pickBody")}</p>
      <div class="cards">
        {#each TEMPLATE_IDS as id (id)}
          <button class="card" onclick={() => onpick?.(id)}>
            {#if id === RECOMMENDED_TEMPLATE}
              <span class="badge">{t("templates.recommended")}</span>
            {/if}
            <strong>{t(`templates.${id}.name`)}</strong>
            <span class="desc">{t(`templates.${id}.desc`)}</span>
          </button>
        {/each}
      </div>
      <button class="cancel" onclick={onclose}>{t("templates.cancel")}</button>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .modal {
    max-width: 640px;
    width: 90%;
    padding: var(--az-space-4);
    background: var(--az-color-panel-bg);
    border: 1px solid var(--az-color-border);
    border-radius: var(--az-radius);
    color: var(--az-color-text);
  }
  h2 {
    margin: 0 0 var(--az-space-1);
  }
  .body {
    margin: 0 0 var(--az-space-3);
    color: var(--az-color-text-muted);
    font-size: 0.9rem;
  }
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: var(--az-space-3);
  }
  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--az-space-1);
    text-align: left;
    padding: var(--az-space-3);
    border: 1px solid var(--az-color-border);
    border-radius: var(--az-radius);
    background: var(--az-color-editor-bg);
    color: var(--az-color-text);
    cursor: pointer;
    font-family: inherit;
  }
  .card:hover,
  .card:focus-visible {
    border-color: var(--az-color-accent);
    outline: none;
  }
  .badge {
    align-self: flex-start;
    padding: 2px 6px;
    border-radius: var(--az-radius);
    background: var(--az-color-accent);
    color: var(--az-color-accent-contrast);
    font-size: 0.7rem;
    font-weight: 700;
  }
  .desc {
    color: var(--az-color-text-muted);
    font-size: 0.85rem;
  }
  .cancel {
    margin-top: var(--az-space-3);
    padding: var(--az-space-1) var(--az-space-3);
    border-radius: var(--az-radius);
    border: 1px solid var(--az-color-border);
    background: none;
    color: var(--az-color-text);
    cursor: pointer;
    font-family: inherit;
  }
</style>
