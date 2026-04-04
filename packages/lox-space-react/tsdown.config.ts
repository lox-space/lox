import { defineConfig } from "tsdown";

export default defineConfig({
  platform: "browser",
  deps: {
    neverBundle: [/^@lox-space\//, "react", "react-dom", "three"],
  },
});
