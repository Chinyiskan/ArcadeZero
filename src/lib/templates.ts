// Plantillas de "Nuevo proyecto" (PLAN.md §5/§11 Fase 2 y 4). Nombres de
// carpeta reales en `templates/` (ver `src-tauri/src/project.rs`).
export type TemplateId = "en-blanco" | "mi-primer-sprite" | "plataformas-basico";

export const TEMPLATE_IDS: TemplateId[] = ["en-blanco", "mi-primer-sprite", "plataformas-basico"];

/** Plantilla recomendada para quien recien abre ArcadeZero (PLAN.md §6.6):
 * ya trae un Actor moviendose con las flechas, lista para pulsar Jugar. */
export const RECOMMENDED_TEMPLATE: TemplateId = "mi-primer-sprite";

export function isTemplateId(id: string): id is TemplateId {
  return (TEMPLATE_IDS as string[]).includes(id);
}
