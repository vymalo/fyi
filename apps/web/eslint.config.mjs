import nextConfig from "eslint-config-next";
import sharedConfig from "@vymalo/config/eslint.config.cjs";

export default [
  ...sharedConfig,
  ...nextConfig,
  {
    ignores: ["node_modules", ".next", "dist"],
  },
];
