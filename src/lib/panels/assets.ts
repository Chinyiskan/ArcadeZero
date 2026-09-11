// Panel de assets (PLAN.md §5, §6.1, §11 Fase 2): tipos y helpers puros,
// separados del componente Svelte para poder testear sin montar UI.

export type AssetKind = "images" | "sounds" | "music";

export const ASSET_KINDS: AssetKind[] = ["images", "sounds", "music"];

/** Espejo de `allowed_extensions()` en src-tauri/src/assets.rs — usado para
 * filtrar el selector de archivos al importar. */
export const ASSET_EXTENSIONS: Record<AssetKind, string[]> = {
  images: ["png", "gif", "jpg", "jpeg"],
  sounds: ["wav", "ogg"],
  music: ["ogg", "mp3"],
};

export function assetKindLabel(kind: AssetKind): string {
  return { images: "Imágenes", sounds: "Sonidos", music: "Música" }[kind];
}

/** El backend ya devuelve la lista ordenada, pero re-ordenar en el
 * frontend nos deja resistentes a que eso cambie y es gratis. */
export function sortAssetNames(names: string[]): string[] {
  return [...names].sort((a, b) => a.localeCompare(b, "es"));
}

/** Estado de confirmación de borrado: como mucho un asset pendiente a la
 * vez (evita diálogos apilados). `null` = nada pendiente. */
export type PendingDelete = { kind: AssetKind; filename: string } | null;

export function requestDelete(kind: AssetKind, filename: string): PendingDelete {
  return { kind, filename };
}

/** ¿Este asset es el que está pidiendo confirmación ahora mismo? */
export function isPending(pending: PendingDelete, kind: AssetKind, filename: string): boolean {
  return pending !== null && pending.kind === kind && pending.filename === filename;
}
