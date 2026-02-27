// vite.config.ts — Vite bundler configuration
//
// Key settings to configure here:
//   - `plugins`: include @vitejs/plugin-react for JSX transform
//   - `resolve.alias`: add "@" → "./src" shortcut so imports look clean
//   - `server.proxy`: proxy "/ws" to "ws://localhost:9001" so the dev server
//     forwards WebSocket connections to the Rust backend without CORS issues
//     (alternatively you can use the full URL and rely on the CORS config in ws_server.rs)

// TODO: implement
export default {}
