import { tweened } from "svelte/motion";
import { cubicOut } from "svelte/easing";
import type { Action } from "svelte/action";
import { PerfectCursor } from "perfect-cursors";

export type SlideParams = {
  x: number;
  y: number;
  center: number[];
  zoom: number;
  immediate?: boolean;
};

/** An action for tweened transitions with global transformations. */
export const slide: Action<HTMLElement, SlideParams> = (node, params) => {
  let center = params?.center ?? [0, 0];
  let zoom = params?.zoom ?? 1;

  const pos = { x: params?.x ?? 0, y: params?.y ?? 0 };
  const spos = tweened(pos, { duration: 150, easing: cubicOut });

  const disposeSub = spos.subscribe((pos) => {
    node.style.transform = `scale(${(zoom * 100).toFixed(3)}%)
      translate3d(${pos.x - center[0]}px, ${pos.y - center[1]}px, 0)`;
  });

  return {
    update(params) {
      center = params?.center ?? [0, 0];
      zoom = params?.zoom ?? 1;
      const pos = { x: params?.x ?? 0, y: params?.y ?? 0 };
      spos.set(pos, { duration: params.immediate ? 0 : 150 });
    },

    destroy() {
      disposeSub();
      node.style.transform = "";
    },
  };
};

/**
 * An action using perfect-cursors to transition an element.
 *
 * The transitions are really smooth geometrically, but they seem to introduce
 * too much noticeable delay. Keeping this function for reference.
 */
export const slideCursor: Action<HTMLElement, SlideParams> = (node, params) => {
  const pos = params ?? { x: 0, y: 0 };

  const pc = new PerfectCursor(([x, y]: number[]) => {
    node.style.transform = `translate3d(${x}px, ${y}px, 0)`;
  });
  pc.addPoint([pos.x, pos.y]);

  return {
    update(params) {
      const pos = params ?? { x: 0, y: 0 };
      pc.addPoint([pos.x, pos.y]);
    },

    destroy() {
      pc.dispose();
      node.style.transform = "";
    },
  };
};
