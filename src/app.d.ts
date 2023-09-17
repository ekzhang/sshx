/// <reference types="@sveltejs/kit" />

// Injected by vite.config.ts
declare const __APP_VERSION__: string;

// See https://kit.svelte.dev/docs/types#the-app-namespace
// for information about these interfaces
declare namespace App {
  // interface Locals {}
  // interface Platform {}
  // interface Session {}
  // interface Stuff {}
}

// Type declarations for external libraries.
declare module "fontfaceobserver";
