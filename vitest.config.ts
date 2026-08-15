import path from "node:path";

import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// Testes de comportamento do frontend (SPEC-008).
//
// Ambiente jsdom com Testing Library. Os testes consultam por papel, texto e
// estado — nunca por classe CSS ou estrutura interna — para que refatoração
// visual não quebre a suíte.
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test/setup.ts"],
    include: ["src/**/*.test.{ts,tsx}"],
    restoreMocks: true,
    clearMocks: true,
  },
});
