import { describe, expect, it } from "vitest";
import {
  ASSET_EXTENSIONS,
  ASSET_KINDS,
  isPending,
  requestDelete,
  sortAssetNames,
} from "./assets";

describe("sortAssetNames", () => {
  it("ordena alfabeticamente sin mutar el original", () => {
    const original = ["nave.png", "alien.png", "explosion.png"];
    const sorted = sortAssetNames(original);
    expect(sorted).toEqual(["alien.png", "explosion.png", "nave.png"]);
    expect(original).toEqual(["nave.png", "alien.png", "explosion.png"]);
  });

  it("no revienta con lista vacia", () => {
    expect(sortAssetNames([])).toEqual([]);
  });
});

describe("ASSET_EXTENSIONS", () => {
  it("tiene una entrada por cada AssetKind", () => {
    for (const kind of ASSET_KINDS) {
      expect(ASSET_EXTENSIONS[kind].length).toBeGreaterThan(0);
    }
  });
});

describe("pending delete", () => {
  it("solo un asset pendiente a la vez", () => {
    const pending = requestDelete("images", "nave.png");
    expect(isPending(pending, "images", "nave.png")).toBe(true);
    expect(isPending(pending, "images", "alien.png")).toBe(false);
    expect(isPending(pending, "sounds", "nave.png")).toBe(false);
    expect(isPending(null, "images", "nave.png")).toBe(false);
  });
});
