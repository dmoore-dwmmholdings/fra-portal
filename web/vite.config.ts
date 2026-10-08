import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  // Preact behind the React API: same code, ~60 KB less JavaScript to download.
  resolve: {
    alias: {
      'react-dom/client': 'preact/compat/client',
      'react-dom': 'preact/compat',
      'react/jsx-runtime': 'preact/jsx-runtime',
      'react/jsx-dev-runtime': 'preact/jsx-dev-runtime',
      react: 'preact/compat',
    },
  },
  // `npm run dev` reads the live payload; set FRA_API to point elsewhere (e.g. the hosting emulator).
  server: { proxy: { '/api': { target: process.env.FRA_API ?? 'https://fra-portal.web.app', changeOrigin: true } } },
})
