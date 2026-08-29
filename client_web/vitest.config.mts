import { defineConfig, configDefaults } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": "/src",
    },
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: ["./vitest.setup.ts"],
    exclude: [...configDefaults.exclude, "**/.next/**"],
    coverage: {
      provider: "v8",
      reporter: ["text", "lcov"],
      include: ["src/**/*.ts", "src/**/*.tsx"],
      exclude: [
        "src/app/**",
        "src/**/*.test.ts",
        "src/**/*.test.tsx",
        "vitest.setup.ts",
        "src/**/index.ts",
        "src/**/types.ts",
        "src/mocks/**",
        "src/shared/providers/**",
      ],
    },
  },
});
