import { spring } from "svelte/motion";
import type { Action } from "svelte/action";

export type SlideParams = {
  x: number;
  y: number;
};

/** An action for spring-y transitions with global transformations. */
export const slide: Action<HTMLElement, SlideParams> = (node, params) => {
  const pos = params ?? { x: 0, y: 0 };
  const xpos = spring(pos.x, { stiffness: 0.6, damping: 1.6 });
  const ypos = spring(pos.y, { stiffness: 0.6, damping: 1.6 });

  const callbackX = xpos.subscribe((x) => {
    pos.x = x;
    node.style.transform = `translate3d(${pos.x}px, ${pos.y}px, 0)`;
  });
  const callbackY = ypos.subscribe((y) => {
    pos.y = y;
    node.style.transform = `translate3d(${pos.x}px, ${pos.y}px, 0)`;
  });

  return {
    update(params) {
      const pos = params ?? { x: 0, y: 0 };
      xpos.set(pos.x);
      ypos.set(pos.y);
    },

    destroy() {
      callbackX();
      callbackY();
      node.style.transform = "";
    },
  };
};
