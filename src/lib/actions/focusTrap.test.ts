// @vitest-environment happy-dom
import { describe, expect, it } from "vitest";
import { focusTrap } from "./focusTrap";

function makeModal() {
  const node = document.createElement("div");
  node.innerHTML = `<button id="a">a</button><button id="b">b</button>`;
  document.body.appendChild(node);
  return node;
}

describe("focusTrap", () => {
  it("wraps Tab from the last focusable back to the first", () => {
    const node = makeModal();
    const b = node.querySelector<HTMLElement>("#b")!;
    const a = node.querySelector<HTMLElement>("#a")!;
    const action = focusTrap(node);
    b.focus();
    const event = new KeyboardEvent("keydown", { key: "Tab", cancelable: true });
    node.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
    expect(document.activeElement).toBe(a);
    action.destroy();
    node.remove();
  });

  it("wraps Shift+Tab from the first focusable back to the last", () => {
    const node = makeModal();
    const a = node.querySelector<HTMLElement>("#a")!;
    const b = node.querySelector<HTMLElement>("#b")!;
    const action = focusTrap(node);
    a.focus();
    const event = new KeyboardEvent("keydown", { key: "Tab", shiftKey: true, cancelable: true });
    node.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
    expect(document.activeElement).toBe(b);
    action.destroy();
    node.remove();
  });

  it("restores focus to the previously focused element on destroy", () => {
    const trigger = document.createElement("button");
    document.body.appendChild(trigger);
    trigger.focus();
    const node = makeModal();
    const action = focusTrap(node);
    action.destroy();
    expect(document.activeElement).toBe(trigger);
    node.remove();
    trigger.remove();
  });
});
