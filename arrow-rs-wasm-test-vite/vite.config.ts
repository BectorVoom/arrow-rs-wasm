import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    fs: {
      allow: ['..']
    },
    middlewareMode: false
  },
  assetsInclude: ['**/*.wasm'],
  optimizeDeps: {
    exclude: ['arrow-rs-wasm']
  }
})
