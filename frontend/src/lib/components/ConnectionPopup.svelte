<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { AIRPODS4_LEFT_IMG, AIRPODS4_RIGHT_IMG, AIRPODS4_CASE_IMG } from '../assets';

  export let deviceName = "Airpod 4 ANC của KXT";
  export let leftBattery = 100;
  export let rightBattery = 83;
  export let caseBattery = 62;
  export let leftCharging = true;
  export let rightCharging = true;
  export let caseCharging = false;

  function closePopup() {
    invoke('close_popup').catch(() => {});
  }

  onMount(() => {
    // Tự động tắt sau 7 giây
    const timer = setTimeout(() => {
      closePopup();
    }, 7000);

    return () => clearTimeout(timer);
  });
</script>

<div class="popup-wrapper">
  <div class="apple-bottom-popup relative">
    
    <!-- Auto-dismiss countdown bar -->
    <div class="auto-dismiss-bar">
      <div class="auto-dismiss-fill"></div>
    </div>

    <!-- Tiêu đề thiết bị -->
    <div class="popup-title">{deviceName}</div>

    <!-- Hàng hiển thị: Cột Trái = Tai L & R, Cột Phải = Hộp sạc -->
    <div class="popup-devices-row">
      
      <!-- Cặp tai nghe L & R -->
      <div class="buds-container">
        
        <!-- Tai Trái (L) -->
        <div class="bud-unit">
          <div class="render-box">
            <img src={AIRPODS4_LEFT_IMG} alt="AirPods 4 Left" class="bud-img" />
          </div>
          <div class="side-badge">L</div>
          
          <!-- Apple Exact SF Symbol Battery SVG Capsule -->
          <div class="battery-capsule-row">
            <svg width="24" height="12" viewBox="0 0 25 12" fill="none" class="battery-svg">
              <rect x="0.75" y="0.75" width="20.5" height="10.5" rx="3.5" stroke="#1d1d1f" stroke-width="1.5"/>
              <path d="M22.5 4C23.3 4 23.5 4.5 23.5 6C23.5 7.5 23.3 8 22.5 8" fill="#1d1d1f"/>
              <rect x="2.5" y="2.5" width={17 * (leftBattery / 100)} height="7" rx="2" fill="#34c759"/>
            </svg>
            {#if leftCharging}
              <svg class="bolt-svg" viewBox="0 0 24 24" fill="#34c759"><path d="M11.5 2L4 13h6l-2 9 9.5-12h-6.5l2-8z"/></svg>
            {/if}
          </div>
          <div class="battery-text">{leftBattery}%</div>
        </div>

        <!-- Tai Phải (R) -->
        <div class="bud-unit">
          <div class="render-box">
            <img src={AIRPODS4_RIGHT_IMG} alt="AirPods 4 Right" class="bud-img" />
          </div>
          <div class="side-badge">R</div>
          
          <!-- Apple Exact SF Symbol Battery SVG Capsule -->
          <div class="battery-capsule-row">
            <svg width="24" height="12" viewBox="0 0 25 12" fill="none" class="battery-svg">
              <rect x="0.75" y="0.75" width="20.5" height="10.5" rx="3.5" stroke="#1d1d1f" stroke-width="1.5"/>
              <path d="M22.5 4C23.3 4 23.5 4.5 23.5 6C23.5 7.5 23.3 8 22.5 8" fill="#1d1d1f"/>
              <rect x="2.5" y="2.5" width={17 * (rightBattery / 100)} height="7" rx="2" fill="#34c759"/>
            </svg>
            {#if rightCharging}
              <svg class="bolt-svg" viewBox="0 0 24 24" fill="#34c759"><path d="M11.5 2L4 13h6l-2 9 9.5-12h-6.5l2-8z"/></svg>
            {/if}
          </div>
          <div class="battery-text">{rightBattery}%</div>
        </div>

      </div>

      <!-- Hộp sạc (Case) -->
      <div class="case-unit">
        <div class="case-render-box">
          <img src={AIRPODS4_CASE_IMG} alt="AirPods 4 Case" class="case-img" />
        </div>
        
        <!-- Apple Exact Battery Capsule Hộp Sạc -->
        <div class="battery-capsule-row">
          <svg width="24" height="12" viewBox="0 0 25 12" fill="none" class="battery-svg">
            <rect x="0.75" y="0.75" width="20.5" height="10.5" rx="3.5" stroke="#1d1d1f" stroke-width="1.5"/>
            <path d="M22.5 4C23.3 4 23.5 4.5 23.5 6C23.5 7.5 23.3 8 22.5 8" fill="#1d1d1f"/>
            <rect x="2.5" y="2.5" width={17 * (caseBattery / 100)} height="7" rx="2" fill="#34c759"/>
          </svg>
          {#if caseCharging}
            <svg class="bolt-svg" viewBox="0 0 24 24" fill="#34c759"><path d="M11.5 2L4 13h6l-2 9 9.5-12h-6.5l2-8z"/></svg>
          {/if}
        </div>
        <div class="battery-text">{caseBattery}%</div>
      </div>

    </div>

    <!-- Nút "Xong" (Done) chuẩn Apple -->
    <button class="popup-done-pill" on:click={closePopup}>
      Xong
    </button>

  </div>
</div>

<style>
  .popup-wrapper {
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: flex-end;
    justify-content: flex-end;
    padding: 24px;
    box-sizing: border-box;
    background: transparent;
  }

  .apple-bottom-popup {
    width: 320px;
    background: rgba(255, 255, 255, 0.94);
    backdrop-filter: blur(50px) saturate(200%);
    -webkit-backdrop-filter: blur(50px) saturate(200%);
    border-radius: 28px;
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.35), 0 4px 16px rgba(0, 0, 0, 0.12);
    border: 1px solid rgba(255, 255, 255, 0.8);
    padding: 18px 18px 20px 18px;
    display: flex;
    flex-direction: column;
    align-items: center;
    color: #1d1d1f;
    position: relative;
    overflow: hidden;
    animation: slideInRight 0.35s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes slideInRight {
    from { transform: translateX(110%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }

  .auto-dismiss-bar {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: rgba(0, 0, 0, 0.08);
  }

  .auto-dismiss-fill {
    height: 100%;
    background: #000000;
    width: 100%;
    animation: dismissCountdown 7s linear forwards;
  }

  @keyframes dismissCountdown {
    from { width: 100%; }
    to { width: 0%; }
  }

  .popup-title {
    font-size: 15.5px;
    font-weight: 600;
    letter-spacing: -0.3px;
    color: #1d1d1f;
    margin-bottom: 14px;
    text-align: center;
  }

  .popup-devices-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    width: 100%;
    padding: 0 4px;
  }

  .buds-container {
    display: flex;
    gap: 14px;
  }

  .bud-unit {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 58px;
  }

  .render-box {
    height: 86px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .bud-img {
    height: 76px;
    object-fit: contain;
    filter: drop-shadow(0 4px 8px rgba(0,0,0,0.12));
  }

  .side-badge {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #8e8e93;
    color: #ffffff;
    font-size: 9px;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-top: 4px;
    margin-bottom: 6px;
  }

  .case-unit {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 120px;
    margin-left: 8px;
  }

  .case-render-box {
    height: 112px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .case-img {
    width: 110px;
    object-fit: contain;
    filter: drop-shadow(0 6px 12px rgba(0,0,0,0.12));
  }

  .battery-capsule-row {
    display: flex;
    align-items: center;
    gap: 3px;
    margin-bottom: 2px;
  }

  .battery-svg {
    flex-shrink: 0;
  }

  .bolt-svg {
    width: 10px;
    height: 10px;
    flex-shrink: 0;
  }

  .battery-text {
    font-size: 11.5px;
    font-weight: 600;
    color: rgba(0, 0, 0, 0.85);
  }

  .popup-done-pill {
    margin-top: 18px;
    width: 100%;
    padding: 10px 0;
    border-radius: 9999px;
    background: #000000;
    color: #ffffff;
    font-size: 13.5px;
    font-weight: 600;
    border: none;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .popup-done-pill:hover {
    background: #2c2c2e;
  }

  .popup-done-pill:active {
    transform: scale(0.98);
  }
</style>
