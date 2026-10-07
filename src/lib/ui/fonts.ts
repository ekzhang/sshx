/** Fallback stack appended to every terminal font, matching Tailwind's mono. */
const fallback =
  'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace';

/**
 * Font families available for terminal text. Only Fira Code is bundled with
 * the app; the others are used if installed locally, else the fallback applies.
 */
const fonts = {
  "Fira Code": `"Fira Code VF", ${fallback}`,
  // Nerd Font variants include icon glyphs used by prompts like Starship.
  "FiraCode Nerd Font": `"FiraCode Nerd Font Mono", ${fallback}`,
  "D2Coding Nerd Font": `"D2CodingLigature Nerd Font Mono", ${fallback}`,
  "JetBrains Mono": `"JetBrains Mono", ${fallback}`,
  "Cascadia Code": `"Cascadia Code", ${fallback}`,
  "Source Code Pro": `"Source Code Pro", ${fallback}`,
  "SF Mono": `"SF Mono", ${fallback}`,
  Menlo: `Menlo, ${fallback}`,
  Consolas: `Consolas, ${fallback}`,
  "System Default": "monospace",
};

export type FontName = keyof typeof fonts;

export const defaultFont: FontName = "Fira Code";

export default fonts;
