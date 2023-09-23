const ISECT_W = 752;
const ISECT_H = 515;
const ISECT_PAD = 16;

type ExistingTerminal = {
  x: number;
  y: number;
  width: number;
  height: number;
};

/** Choose a position for a new terminal that does not intersect existing ones. */
export function arrangeNewTerminal(existing: ExistingTerminal[]) {
  if (existing.length === 0) {
    return { x: 0, y: 0 };
  }

  const startX = 100 * (Math.random() - 0.5);
  const startY = 60 * (Math.random() - 0.5);

  for (let i = 0; ; i++) {
    const t = 1.94161103872 * i;
    const x = Math.round(startX + 8 * i * Math.cos(t));
    const y = Math.round(startY + 8 * i * Math.sin(t));
    let ok = true;
    for (const box of existing) {
      if (
        isect(x, x + ISECT_W, box.x, box.x + box.width) &&
        isect(y, y + ISECT_H, box.y, box.y + box.height)
      ) {
        ok = false;
        break;
      }
    }
    if (ok) {
      return { x, y };
    }
  }
}

function isect(s1: number, e1: number, s2: number, e2: number): boolean {
  return s1 - ISECT_PAD < e2 && e1 + ISECT_PAD > s2;
}
