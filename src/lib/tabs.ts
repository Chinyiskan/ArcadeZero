// Pestañas de previsualización (PLAN.md §6.1): `main.py` es fija y no
// cerrable, y clic en una imagen del panel de assets abre/activa una
// pestaña de solo-lectura. Lógica pura, sin Svelte, para poder testear
// abrir/activar/cerrar/no-duplicar sin montar UI.

export type Tab =
  | { id: "main"; kind: "main"; label: "main.py" }
  | { id: string; kind: "image"; label: string; filename: string };

export const MAIN_TAB: Tab = { id: "main", kind: "main", label: "main.py" };

export function imageTabId(filename: string): string {
  return `image:${filename}`;
}

export function initialTabs(): Tab[] {
  return [MAIN_TAB];
}

/** Abre la pestaña de una imagen, o la activa si ya está abierta (nunca duplica). */
export function openImageTab(
  tabs: Tab[],
  filename: string,
): { tabs: Tab[]; activeId: string } {
  const id = imageTabId(filename);
  if (tabs.some((tab) => tab.id === id)) {
    return { tabs, activeId: id };
  }
  const tab: Tab = { id, kind: "image", label: filename, filename };
  return { tabs: [...tabs, tab], activeId: id };
}

export function activateTab(tabs: Tab[], id: string): string {
  return tabs.some((tab) => tab.id === id) ? id : tabs[0]?.id ?? MAIN_TAB.id;
}

/** Cierra una pestaña cerrable (todas menos `main`). Si estaba activa, activa
 * la que quedó a su izquierda (o `main` si no queda ninguna). */
export function closeTab(
  tabs: Tab[],
  activeId: string,
  id: string,
): { tabs: Tab[]; activeId: string } {
  if (id === "main") return { tabs, activeId };
  const index = tabs.findIndex((tab) => tab.id === id);
  if (index === -1) return { tabs, activeId };

  const nextTabs = tabs.filter((tab) => tab.id !== id);
  if (activeId !== id) return { tabs: nextTabs, activeId };

  const fallback = tabs[index - 1] ?? nextTabs[0] ?? MAIN_TAB;
  return { tabs: nextTabs, activeId: fallback.id };
}
