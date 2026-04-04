// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

// https://astro.build/config
export default defineConfig({
  integrations: [
    starlight({
      title: "Lox",
      social: [
        {
          icon: "github",
          label: "Lox Space",
          href: "https://github.com/lox-space/lox",
        },
      ],
      sidebar: [
        {
          label: "Start Here",
          items: [
            { label: "Getting Started", slug: "getting-started" },
            { label: "Tutorial", slug: "tutorial" },
          ],
        },
        {
          label: "Guides",
          items: [
            // Each item here is one entry in the navigation menu.
            { label: "Example Guide", slug: "guides/example" },
          ],
        },
        {
          label: "Reference",
          items: [
            { label: "Rust (docs.rs)", link: "https://docs.rs/lox-space" },
            { label: "Python", link: "https://python.lox-space.org" },
          ],
        },
      ],
    }),
  ],
});
