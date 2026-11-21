import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const here = path.dirname(fileURLToPath(import.meta.url));
const frontend = path.resolve(here, '..');
const source = path.resolve(frontend, '..', 'preview.html');
let html = readFileSync(source, 'utf8');

html = html.replace(
  '<script src="https://www.gstatic.com/antigravity/web/dev/tailwindcss.min.js"></script>',
  '<link rel="stylesheet" href="/preview-tailwind.css" />',
);

const scriptStart = html.lastIndexOf('  <script>');
const scriptEnd = html.lastIndexOf('  </script>');
if (scriptStart < 0 || scriptEnd < scriptStart) {
  throw new Error('Không tìm thấy script cuối trong preview.html');
}
html = html.slice(0, scriptStart) + html.slice(scriptEnd + '  </script>'.length);
html = html.replace('</head>', `
  <link rel="icon" type="image/png" href="/lotusair-icon.png" />
  <style>
    /* Tắt CSS backdrop-filter giả (tốn ~180MB GPU RAM) để nhường cho Windows 11 Native DWM Acrylic Glass (0MB RAM!) */
    *, *::before, *::after {
      backdrop-filter: none !important;
      -webkit-backdrop-filter: none !important;
    }

    html, body {
      background: transparent !important;
      overflow: hidden;
      margin: 0;
      padding: 0;
      width: 100vw;
      height: 100vh;
    }

    .desktop-scene {
      background: transparent !important;
      min-height: 100vh;
      height: 100vh;
      width: 100vw;
      padding: 0 !important;
      overflow: hidden;
      justify-content: center;
      align-items: center;
    }

    /* Dark macOS Sequoia Native Acrylic Glass (để xuyên trọn vẹn kính mờ Windows 11 DWM Acrylic phía sau!) */
    .mac-control-panel {
      width: 100vw !important;
      height: 100vh !important;
      max-height: 100vh !important;
      border-radius: 10px !important;
      background: linear-gradient(155deg, rgba(255, 255, 255, 0.08) 0%, rgba(16, 16, 24, 0.24) 100%) !important;
      border: 1px solid rgba(255, 255, 255, 0.24) !important;
      box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.34) !important;
      overflow-y: auto;
      scrollbar-width: none;
      will-change: transform, opacity;
      animation: lotusFlyoutIn 0.20s cubic-bezier(0.22, 1, 0.36, 1) both !important;
    }
    .mac-control-panel::-webkit-scrollbar { display: none; }

    /* Tăng nhẹ tương phản các thẻ con bên trong để chữ sắc nét cả trên nền sáng (YouTube) lẫn nền tối */
    .top-split-card, .volume-capsule-card, .output-list-card, .segmented-control, .segmented-stack-track {
      background: rgba(12, 12, 18, 0.32) !important;
      border: 1px solid rgba(255, 255, 255, 0.14) !important;
    }

    /* Vô hiệu hóa các mục điều khiển AirPods khi chưa kết nối Windows */
    .airpods-control-disabled {
      opacity: 0.36 !important;
      pointer-events: none !important;
      filter: grayscale(0.45);
      transition: opacity 0.2s ease, filter 0.2s ease;
    }
    .airpods-control-disabled .segmented-stack-btn.active,
    .airpods-control-disabled .conv-btn.active {
      background: rgba(255, 255, 255, 0.16) !important;
      color: rgba(255, 255, 255, 0.65) !important;
      box-shadow: none !important;
    }
    .airpods-offline-tag {
      font-size: 9.5px;
      font-weight: 500;
      color: rgba(255, 255, 255, 0.42);
      text-transform: none;
      letter-spacing: 0;
    }

    .mac-control-panel.re-enter {
      animation: lotusFlyoutIn 0.20s cubic-bezier(0.22, 1, 0.36, 1) both !important;
    }

    .settings-window-panel {
      width: 100vw !important;
      max-width: 100vw !important;
      height: 100vh !important;
      max-height: 100vh !important;
      border-radius: 10px !important;
      background: linear-gradient(160deg, rgba(255, 255, 255, 0.06) 0%, rgba(16, 16, 24, 0.34) 100%) !important;
      border: 1px solid rgba(255, 255, 255, 0.22) !important;
      box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.28) !important;
      animation: lotusFlyoutIn 0.22s cubic-bezier(0.22, 1, 0.36, 1) both !important;
    }

    /* Apple tactile micro-interactions (pure GPU transform/opacity) */
    .seg-btn, .mode-pill, .output-item-row, .settings-row, .top-split-card {
      transition: transform 0.22s cubic-bezier(0.22, 1, 0.36, 1), background-color 0.2s ease, box-shadow 0.22s ease, color 0.18s ease !important;
    }
    .seg-btn:active, .mode-pill:active, .output-item-row:active {
      transform: scale(0.955) !important;
    }
    .output-item-row:hover {
      transform: translate3d(2px, 0, 0);
    }
    .ring-bolt-sm {
      animation: lotusBoltPulse 2.2s ease-in-out infinite;
    }

    .view-switcher-bar { display: none !important; }
    body.control-view #settings-window-panel, body.control-view #bottom-right-container { display: none !important; }
    body.settings-view #mac-control-panel, body.settings-view #bottom-right-container { display: none !important; }
    body.settings-view .desktop-scene { overflow-y: auto; scrollbar-width: thin; }
    body.settings-view .settings-window-panel { display: flex !important; }
    #bottom-right-container { display: none; }

    .app-state-banner {
      position: fixed;
      z-index: 1000;
      left: 16px;
      right: 16px;
      bottom: 12px;
      padding: 8px 12px;
      border-radius: 12px;
      background: rgba(10, 132, 255, 0.88);
      border: 1px solid rgba(255, 255, 255, 0.28);
      color: white;
      font-size: 11.5px;
      line-height: 1.35;
      text-align: center;
      animation: lotusCardCascade 0.24s cubic-bezier(0.22, 1, 0.36, 1) both;
    }
    .app-state-banner.error { background: rgba(255, 69, 58, 0.9); border-color: rgba(255, 255, 255, 0.28); }

    /* Bright Apple iOS 18 Liquid Glass Popup */
    body.popup-view { background: transparent !important; overflow: hidden; margin: 0; padding: 0; }
    body.popup-view .desktop-scene { background: transparent !important; padding: 0 !important; min-height: 100vh; height: 100vh; width: 100vw; overflow: hidden; }
    body.popup-view .view-switcher-bar, body.popup-view #mac-control-panel, body.popup-view #settings-window-panel, body.popup-view .app-state-banner { display: none !important; }
    body.popup-view #bottom-right-container {
      display: flex !important;
      position: fixed;
      inset: 0;
      align-items: stretch;
      justify-content: stretch;
      bottom: auto;
      right: auto;
      padding: 0;
      will-change: transform, opacity;
    }
    body.popup-view .apple-bottom-popup {
      width: 100vw !important;
      height: 100vh !important;
      max-width: 100vw !important;
      justify-content: space-between;
      box-sizing: border-box;
      border-radius: 10px !important;
      background: linear-gradient(160deg, rgba(255, 255, 255, 0.42) 0%, rgba(240, 244, 252, 0.52) 100%) !important;
      box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.94) !important;
      border: 1px solid rgba(255, 255, 255, 0.78) !important;
    }

    /* Liquid Glass specular light shimmer sweep on popup open */
    body.popup-view .apple-bottom-popup::after {
      content: "";
      position: absolute;
      inset: 0;
      background: linear-gradient(115deg, transparent 22%, rgba(255, 255, 255, 0.55) 48%, transparent 72%);
      transform: translate3d(-120%, 0, 0);
      pointer-events: none;
      animation: lotusGlassShimmer 1.15s cubic-bezier(0.22, 1, 0.36, 1) 0.12s 1 forwards;
    }

    /* Elastic Apple Spring Popup Enter / Exit */
    .popup-spring-in {
      animation: lotusPopupSpringIn 0.52s cubic-bezier(0.22, 1, 0.36, 1) forwards !important;
    }
    .popup-spring-out {
      animation: lotusPopupSpringOut 0.22s cubic-bezier(0.4, 0, 1, 1) forwards !important;
    }

    /* 3D Levitating AirPods + Case stagger entrance & gentle breathing float */
    body.popup-view .apple-bottom-popup img {
      will-change: transform;
      filter: drop-shadow(0 8px 12px rgba(0, 0, 0, 0.16));
    }
    body.popup-view .apple-bottom-popup img[alt*="Left"] {
      animation: lotusPodEnter 0.55s cubic-bezier(0.22, 1, 0.36, 1) 0.06s both,
                 lotusFloatL 3.4s ease-in-out 0.62s infinite;
    }
    body.popup-view .apple-bottom-popup img[alt*="Right"] {
      animation: lotusPodEnter 0.55s cubic-bezier(0.22, 1, 0.36, 1) 0.14s both,
                 lotusFloatR 3.4s ease-in-out 0.85s infinite;
    }
    body.popup-view .apple-bottom-popup img[alt*="Case"],
    body.popup-view .apple-bottom-popup img[alt*="Hộp"] {
      animation: lotusPodEnter 0.58s cubic-bezier(0.22, 1, 0.36, 1) 0.20s both,
                 lotusFloatCase 3.8s ease-in-out 0.95s infinite;
    }

    /* Pure GPU countdown progress bar (0% layout work) */
    .auto-dismiss-fill {
      width: 100% !important;
      transform-origin: left center;
      will-change: transform;
      background: linear-gradient(90deg, #1d1d1f 0%, #52525b 100%) !important;
      animation: lotusGpuCountdown 6.5s linear forwards !important;
    }

    @keyframes lotusFlyoutIn {
      0% { opacity: 0; transform: translate3d(0, 14px, 0) scale(0.96); }
      100% { opacity: 1; transform: translate3d(0, 0, 0) scale(1); }
    }
    @keyframes lotusCardCascade {
      0% { opacity: 0; transform: translate3d(0, 8px, 0); }
      100% { opacity: 1; transform: translate3d(0, 0, 0); }
    }
    @keyframes lotusPopupSpringIn {
      0% { opacity: 0; transform: translate3d(0, 34px, 0) scale(0.90); }
      65% { opacity: 1; transform: translate3d(0, -3px, 0) scale(1.012); }
      100% { opacity: 1; transform: translate3d(0, 0, 0) scale(1); }
    }
    @keyframes lotusPopupSpringOut {
      0% { opacity: 1; transform: translate3d(0, 0, 0) scale(1); }
      100% { opacity: 0; transform: translate3d(0, 24px, 0) scale(0.94); }
    }
    @keyframes lotusPodEnter {
      0% { opacity: 0; transform: translate3d(0, 16px, 0) scale(0.84); }
      70% { opacity: 1; transform: translate3d(0, -3px, 0) scale(1.03); }
      100% { opacity: 1; transform: translate3d(0, 0, 0) scale(1); }
    }
    @keyframes lotusFloatL {
      0%, 100% { transform: translate3d(0, 0, 0) rotate(0deg); }
      50% { transform: translate3d(0, -4.5px, 0) rotate(-1.6deg); }
    }
    @keyframes lotusFloatR {
      0%, 100% { transform: translate3d(0, 0, 0) rotate(0deg); }
      50% { transform: translate3d(0, -4.5px, 0) rotate(1.6deg); }
    }
    @keyframes lotusFloatCase {
      0%, 100% { transform: translate3d(0, 0, 0); }
      50% { transform: translate3d(0, -3px, 0); }
    }
    @keyframes lotusGlassShimmer {
      0% { transform: translate3d(-120%, 0, 0); }
      100% { transform: translate3d(130%, 0, 0); }
    }
    @keyframes lotusGpuCountdown {
      0% { transform: scaleX(1); }
      100% { transform: scaleX(0); }
    }
    @keyframes lotusBoltPulse {
      0%, 100% { opacity: 1; transform: scale(1); }
      50% { opacity: 0.65; transform: scale(0.92); }
    }
  </style>
</head>`);
html = html.replace('</body>', '  <script type="module" src="/src/preview.ts"></script>\n</body>');
html = html.replace(/[ \t]+(?=\r?$)/gm, '');
writeFileSync(path.join(frontend, 'index.html'), html);
