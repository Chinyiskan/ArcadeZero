import { describe, expect, it } from "vitest";
import { activateTab, closeTab, imageTabId, initialTabs, MAIN_TAB, openImageTab } from "./tabs";

describe("initialTabs", () => {
  it("arranca solo con main.py, no cerrable", () => {
    expect(initialTabs()).toEqual([MAIN_TAB]);
  });
});

describe("openImageTab", () => {
  it("agrega una pestaña nueva y la activa", () => {
    const { tabs, activeId } = openImageTab(initialTabs(), "nave.png");
    expect(tabs).toEqual([MAIN_TAB, { id: imageTabId("nave.png"), kind: "image", label: "nave.png", filename: "nave.png" }]);
    expect(activeId).toBe(imageTabId("nave.png"));
  });

  it("no duplica: reabrir la misma imagen solo la activa", () => {
    const first = openImageTab(initialTabs(), "nave.png");
    const second = openImageTab(first.tabs, "nave.png");
    expect(second.tabs).toHaveLength(2);
    expect(second.activeId).toBe(imageTabId("nave.png"));
  });

  it("varias imagenes distintas conviven", () => {
    const first = openImageTab(initialTabs(), "nave.png");
    const second = openImageTab(first.tabs, "alien.png");
    expect(second.tabs.map((t) => t.label)).toEqual(["main.py", "nave.png", "alien.png"]);
  });
});

describe("activateTab", () => {
  it("activa una pestaña existente", () => {
    const { tabs } = openImageTab(initialTabs(), "nave.png");
    expect(activateTab(tabs, "main")).toBe("main");
    expect(activateTab(tabs, imageTabId("nave.png"))).toBe(imageTabId("nave.png"));
  });

  it("cae a la primera pestaña si el id no existe", () => {
    const tabs = initialTabs();
    expect(activateTab(tabs, "no-existe")).toBe("main");
  });
});

describe("closeTab", () => {
  it("main.py nunca se cierra", () => {
    const tabs = initialTabs();
    const result = closeTab(tabs, "main", "main");
    expect(result.tabs).toEqual(tabs);
    expect(result.activeId).toBe("main");
  });

  it("cerrar una pestaña inactiva no cambia la activa", () => {
    const { tabs } = openImageTab(initialTabs(), "nave.png");
    const result = closeTab(tabs, "main", imageTabId("nave.png"));
    expect(result.tabs).toEqual([MAIN_TAB]);
    expect(result.activeId).toBe("main");
  });

  it("cerrar la pestaña activa activa la de su izquierda", () => {
    let { tabs, activeId } = openImageTab(initialTabs(), "nave.png");
    ({ tabs, activeId } = openImageTab(tabs, "alien.png"));
    const result = closeTab(tabs, activeId, imageTabId("alien.png"));
    expect(result.activeId).toBe(imageTabId("nave.png"));
    expect(result.tabs.map((t) => t.label)).toEqual(["main.py", "nave.png"]);
  });

  it("cerrar la unica pestaña de imagen activa vuelve a main", () => {
    const { tabs, activeId } = openImageTab(initialTabs(), "nave.png");
    const result = closeTab(tabs, activeId, imageTabId("nave.png"));
    expect(result.activeId).toBe("main");
    expect(result.tabs).toEqual([MAIN_TAB]);
  });
});
