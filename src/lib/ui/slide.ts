import { spring } from "svelte/motion";
import type { Action } from "svelte/action";
import { PerfectCursor } from "perfect-cursors";

export type SlideParams = {
  x: number;
  y: number;
};

/** An action for spring-y transitions with global transformations. */
export const slide: Action<HTMLElement, SlideParams> = (node, params) => {
  const pos = params ?? { x: 0, y: 0 };
  const spos = spring(pos, { stiffness: 0.6, damping: 1.6 });

  const disposeSub = spos.subscribe((pos) => {
    node.style.transform = `translate3d(${pos.x}px, ${pos.y}px, 0)`;
  });

  return {
    update(params) {
      const pos = params ?? { x: 0, y: 0 };
      spos.set(pos);
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
