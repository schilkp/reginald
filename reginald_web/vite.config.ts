import path from "path"
import tailwindcss from "@tailwindcss/vite"
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import topLevelAwait from "vite-plugin-top-level-await";
import wasm from "vite-plugin-wasm";
import { viteStaticCopy } from 'vite-plugin-static-copy'

const dev_port = 5173;

var base;
if (process.env.NODE_ENV === "development") {
    base = "http://localhost:" + dev_port
} else {
    base = process.env.TBAND_BASE || "https://schilk.co/reginald";
}
base = base.replace(/\/$/, '') + '/'

console.log("URL base: " + base);

// https://vite.dev/config/
export default defineConfig({
    base: base,
    define: {
        __REGINALD_BASE__: JSON.stringify(base),
        __REGINALD_REPO__: JSON.stringify("https://github.com/schilkp/reginald"),
    },
    plugins: [
        react(),
        wasm(),
        topLevelAwait(),
        tailwindcss(),

        viteStaticCopy({
            targets: [
                {
                    src: '../docs/book/*',
                    dest: 'docs'
                }
            ]
        })
    ],
    resolve: {
        alias: {
            "@": path.resolve(__dirname, "./src"),
        },
    },
})
