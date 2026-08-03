import { nodeResolve } from '@rollup/plugin-node-resolve';
import terser from '@rollup/plugin-terser';
import postcss from 'rollup-plugin-postcss';

const production = process.env.NODE_ENV === 'production';

export default {
  input: {
    map: 'src/js/map.js',
    'maplibre-gl-worker': 'node_modules/maplibre-gl/dist/maplibre-gl-worker.mjs',
  },
  output: {
    dir: 'public/maplibre/',
    format: 'es',
    entryFileNames: '[name].mjs',
    chunkFileNames: '[name].mjs',
    sourcemap: production ? false : true,
    plugins: production ? [terser()] : [],
  },
  plugins: [
    nodeResolve(),
    postcss({
      extract: 'map.css',
      minimize: false,
      sourceMap: false,
    }),
  ],
};
