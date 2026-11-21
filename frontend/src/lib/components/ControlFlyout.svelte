<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';

  let deviceState = {
    name: "AirPods",
    connected: false,
    noise_mode: "off",
    battery: {
      left: { level: null as number | null, is_charging: false },
      right: { level: null as number | null, is_charging: false },
      case: { level: null as number | null, is_charging: false }
    },
    in_ear: {
      left_in_ear: false,
      right_in_ear: false
    },
    features: {
      conversational_awareness: false,
      personalized_volume: false,
      adaptive_audio_blend: 50,
      keep_alive_connection: false,
      spatial_audio_mode: "off"
    }
  };

  let isEditingName = false;
  let tempName = deviceState.name;
  let isTakeoverLoading = false;
  let errorMessage = '';

  function showError(error: unknown) {
    errorMessage = String(error);
  }

  async function loadState() {
    try {
      const state: any = await invoke('get_device_state');
      if (state) {
        deviceState = state;
        tempName = state.name;
      }
    } catch (e) {
      showError(e);
    }
  }

  async function setMode(modeNum: number) {
    try {
      await invoke('set_noise_mode', { mode: modeNum });
      await loadState();
      errorMessage = '';
    } catch (e) {
      showError(e);
    }
  }

  async function setAdaptiveBlend(level: number) {
    try {
      await invoke('set_adaptive_blend', { level });
      await loadState();
      errorMessage = '';
    } catch (e) {
      showError(e);
    }
  }

  async function setSpatialMode(mode: 'off' | 'fixed' | 'headtracked') {
    try {
      await invoke('set_spatial_audio_mode', { mode });
      await loadState();
      errorMessage = '';
    } catch (e) {
      showError(e);
    }
  }

  async function toggleConversationalAwareness() {
    const nextVal = !deviceState.features.conversational_awareness;
    try {
      await invoke('toggle_conversational_awareness', { enabled: nextVal });
      await loadState();
      errorMessage = '';
    } catch (e) {
      showError(e);
    }
  }

  async function toggleKeepAlive() {
    const nextVal = !deviceState.features.keep_alive_connection;
    try {
      await invoke('set_keep_alive', { enabled: nextVal });
      await loadState();
      errorMessage = '';
    } catch (e) {
      showError(e);
    }
  }

  async function saveNewName() {
    if (tempName.trim().length > 0) {
      try {
        await invoke('rename_device', { newName: tempName.trim() });
        await loadState();
        errorMessage = '';
      } catch (e) {
        showError(e);
      }
    }
    isEditingName = false;
  }

  async function forceTakeover() {
    isTakeoverLoading = true;
    try {
      await invoke('force_takeover_audio');
      await loadState();
      errorMessage = '';
    } catch (e) {
      showError(e);
    } finally {
      setTimeout(() => {
        isTakeoverLoading = false;
      }, 700);
    }
  }

  function triggerPopup() {
    invoke('show_connection_popup').catch(() => {});
  }

  onMount(() => {
    loadState();
    const unlisten = listen('device-state-updated', (event: any) => {
      deviceState = event.payload;
    });

    return () => {
      unlisten.then(f => f());
    };
  });
</script>

<div class="control-wrapper">
  <!-- Seamless Liquid Glass Container - KHÔNG BORDER THÔ CỨNG -->
  <div class="seamless-control-card">
    
    <!-- Header: Tên thiết bị + Đổi tên + Nút xem Popup -->
    <div class="control-header">
      <div class="header-left">
        <img class="app-logo" src="/lotusair-icon.png" alt="LotusAir" />
        <span class="status-dot" class:connected={deviceState.connected}></span>
        
        {#if isEditingName}
          <div class="rename-inline">
            <input
              type="text"
              bind:value={tempName}
              on:keydown={(e) => e.key === 'Enter' && saveNewName()}
              class="rename-input"
            />
            <button class="save-name-btn" on:click={saveNewName}>✓</button>
          </div>
        {:else}
          <button class="title-row" on:click={() => isEditingName = true} title="Nhấp để đổi tên AirPods">
            <span class="device-name">{deviceState.name}</span>
            <svg class="edit-icon-svg" viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"/></svg>
          </button>
        {/if}
      </div>

      <button class="glass-icon-btn" on:click={triggerPopup} title="Xem Popup kết nối">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          <path d="M15.91 11.6l-2.16-1.25a1 1 0 00-1.5.86v3.58a1 1 0 001.5.86l2.16-1.25a1 1 0 000-1.74z" />
        </svg>
      </button>
    </div>

    {#if !deviceState.connected}
      <div class="connection-notice" role="status">Chưa có kết nối điều khiển AirPods. Kết nối âm thanh trong Windows chưa đủ để đổi chế độ.</div>
    {/if}
    {#if errorMessage}
      <div class="error-notice" role="alert">{errorMessage}</div>
    {/if}

    <!-- Quick Action: CƯỚP TAI NGHE TỪ IPHONE / KẾT NỐI TỨC THÌ -->
    <div class="takeover-row">
      <button class="takeover-btn" class:loading={isTakeoverLoading} on:click={forceTakeover}>
        {#if isTakeoverLoading}
          <svg class="animate-spin" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21.5 2v6h-6M2.5 22v-6h6"/><path d="M19.07 10.93a8 8 0 0 0-14.14-2.58L2.5 10M4.93 13.07a8 8 0 0 0 14.14 2.58L21.5 14"/></svg>
        {:else}
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="5" y="2" width="14" height="20" rx="3"/><line x1="12" y1="18" x2="12.01" y2="18"/><path d="M12 7l3 3-3 3M15 10H9"/></svg>
        {/if}
        <span>{isTakeoverLoading ? 'Đang chuyển âm thanh...' : 'Chuyển âm thanh sang PC (Cướp từ iPhone)'}</span>
      </button>
    </div>

    <!-- Pin L / R / Case - Thiết kế không viền nhẹ nhàng -->
    <div class="battery-strip">
      <div class="battery-tile">
        <span class="tile-label">Tai Trái</span>
        <span class="tile-value flex items-center justify-center gap-1">
          {deviceState.battery.left.level ?? '--'}% 
          {#if deviceState.battery.left.is_charging}
            <svg class="inline-block text-emerald-600" width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M11.5 2L4 13h6l-2 9 9.5-12h-6.5l2-8z"/></svg>
          {/if}
        </span>
        <span class="tile-sub" class:active={deviceState.in_ear.left_in_ear}>
          {deviceState.connected ? (deviceState.in_ear.left_in_ear ? 'Đang đeo' : 'Ngoài tai') : 'Chưa rõ'}
        </span>
      </div>

      <div class="battery-tile">
        <span class="tile-label">Tai Phải</span>
        <span class="tile-value flex items-center justify-center gap-1">
          {deviceState.battery.right.level ?? '--'}% 
          {#if deviceState.battery.right.is_charging}
            <svg class="inline-block text-emerald-600" width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M11.5 2L4 13h6l-2 9 9.5-12h-6.5l2-8z"/></svg>
          {/if}
        </span>
        <span class="tile-sub" class:active={deviceState.in_ear.right_in_ear}>
          {deviceState.connected ? (deviceState.in_ear.right_in_ear ? 'Đang đeo' : 'Ngoài tai') : 'Chưa rõ'}
        </span>
      </div>

      <div class="battery-tile">
        <span class="tile-label">Hộp Sạc</span>
        <span class="tile-value flex items-center justify-center gap-1">
          {deviceState.battery.case.level ?? '--'}% 
          {#if deviceState.battery.case.is_charging}
            <svg class="inline-block text-emerald-600" width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M11.5 2L4 13h6l-2 9 9.5-12h-6.5l2-8z"/></svg>
          {/if}
        </span>
        <span class="tile-sub" class:active={deviceState.connected}>{deviceState.connected ? 'Sẵn sàng' : 'Chưa rõ'}</span>
      </div>
    </div>

    <!-- 1. KIỂM SOÁT TIẾNG ỒN (NOISE CONTROL) -->
    <div class="section-block">
      <div class="section-title">KIỂM SOÁT TIẾNG ỒN</div>
      <div class="pill-segmented">
        <button
          class="pill-opt"
          class:selected={deviceState.connected && deviceState.noise_mode === 'off'}
          on:click={() => setMode(1)}
        >
          Tắt
        </button>

        <button
          class="pill-opt"
          class:selected={deviceState.connected && deviceState.noise_mode === 'noisecancellation'}
          on:click={() => setMode(2)}
        >
          Chống ồn
        </button>

        <button
          class="pill-opt"
          class:selected={deviceState.connected && deviceState.noise_mode === 'transparency'}
          on:click={() => setMode(3)}
        >
          Xuyên âm
        </button>

        <button
          class="pill-opt"
          class:selected={deviceState.connected && deviceState.noise_mode === 'adaptive'}
          on:click={() => setMode(4)}
        >
          Thích ứng
        </button>
      </div>

      <!-- THANH TRƯỢT TĂNG GIẢM THÍCH ỨNG (CUSTOMIZABLE ADAPTIVE AUDIO SLIDER) -->
      {#if deviceState.connected && deviceState.noise_mode === 'adaptive'}
        <div class="adaptive-slider-card">
          <div class="slider-labels">
            <span>Ít tiếng ồn</span>
            <span class="blend-level">{deviceState.features.adaptive_audio_blend}%</span>
            <span>Nhiều tiếng ồn</span>
          </div>
          <div class="slider-track-wrap">
            <input
              type="range"
              min="0"
              max="100"
              value={deviceState.features.adaptive_audio_blend}
              on:input={(e) => setAdaptiveBlend(parseInt(e.currentTarget.value))}
              class="apple-range"
            />
          </div>
        </div>
      {/if}
    </div>

    <!-- 2. ÂM THANH KHÔNG GIAN (SPATIAL AUDIO: TẮT / CỐ ĐỊNH / THEO DÕI ĐẦU) -->
    <div class="section-block">
      <div class="section-title">ÂM THANH KHÔNG GIAN (SPATIAL AUDIO)</div>
      <div class="pill-segmented">
        <button
          class="pill-opt"
          class:selected={deviceState.connected && deviceState.features.spatial_audio_mode === 'off'}
          on:click={() => setSpatialMode('off')}
        >
          Tắt
        </button>

        <button
          class="pill-opt"
          class:selected={deviceState.connected && deviceState.features.spatial_audio_mode === 'fixed'}
          on:click={() => setSpatialMode('fixed')}
        >
          Cố định
        </button>

        <button
          class="pill-opt"
          class:selected={deviceState.connected && deviceState.features.spatial_audio_mode === 'headtracked'}
          on:click={() => setSpatialMode('headtracked')}
        >
          Theo dõi đầu
        </button>
      </div>
    </div>

    <!-- 3. CÁC TÍNH NĂNG THÔNG MINH & DUY TRÌ KẾT NỐI -->
    <div class="section-block">
      <div class="section-title">CÀI ĐẶT THÔNG MINH</div>
      <div class="seamless-list">
        
        <!-- Luôn duy trì kết nối (Keep-Alive) -->
        <div class="list-item">
          <div class="item-text">
            <div class="item-name">Luôn duy trì kết nối</div>
            <div class="item-desc">Giữ Bluetooth luôn sẵn sàng trong danh sách âm thanh PC</div>
          </div>
          <button
            class="apple-switch"
            class:active={deviceState.features.keep_alive_connection}
            on:click={toggleKeepAlive}
            aria-label="Luôn duy trì kết nối"
            aria-pressed={deviceState.features.keep_alive_connection}
          >
            <div class="switch-knob"></div>
          </button>
        </div>

        <!-- Nhận biết cuộc hội thoại -->
        <div class="list-item">
          <div class="item-text">
            <div class="item-name">Nhận biết cuộc hội thoại</div>
            <div class="item-desc">Tự động hạ âm lượng khi bạn bắt đầu nói</div>
          </div>
          <button
            class="apple-switch"
            class:active={deviceState.features.conversational_awareness}
            on:click={toggleConversationalAwareness}
            aria-label="Nhận biết cuộc hội thoại"
            aria-pressed={deviceState.features.conversational_awareness}
          >
            <div class="switch-knob"></div>
          </button>
        </div>

        <!-- Âm lượng cá nhân hóa -->
        <div class="list-item">
          <div class="item-text">
            <div class="item-name">Âm lượng cá nhân hóa</div>
            <div class="item-desc">Điều chỉnh thông minh theo môi trường xung quanh</div>
          </div>
          <button
            class="apple-switch"
            class:active={deviceState.features.personalized_volume}
            title="Chưa có lệnh điều khiển tính năng này"
            aria-label="Âm lượng cá nhân hóa chưa được hỗ trợ"
            disabled
          >
            <div class="switch-knob"></div>
          </button>
        </div>

      </div>
    </div>

    <!-- Footer -->
    <div class="control-footer">
      <span>LotusAir • Apple Liquid Glass Control Center</span>
    </div>

  </div>
</div>

<style>
  .control-wrapper {
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 10px;
    box-sizing: border-box;
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text", sans-serif;
  }

  /* Seamless Liquid Glass Container - KHÔNG BORDER */
  .seamless-control-card {
    width: 100%;
    max-width: 360px;
    height: 100%;
    max-height: 540px;
    background: linear-gradient(145deg, rgba(255,255,255,.98), rgba(243,247,251,.96));
    border-radius: 32px;
    /* KHÔNG CÓ BORDER THÔ CỨNG */
    border: none !important;
    box-shadow: 
      0 24px 60px rgba(0, 0, 0, 0.25),
      0 4px 16px rgba(0, 0, 0, 0.08),
      inset 0 1px 1px rgba(255, 255, 255, 0.6);
    padding: 22px 20px 16px 20px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 14px;
    color: var(--lg-text-primary);
    overflow-y: auto;
    position: relative;
  }

  /* Header */
  .control-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .app-logo {
    width: 28px;
    height: 28px;
    object-fit: contain;
    flex-shrink: 0;
    filter: drop-shadow(0 2px 3px rgba(0,0,0,.18));
  }

  .connection-notice, .error-notice {
    border-radius: 12px;
    padding: 8px 10px;
    font-size: 11px;
    line-height: 1.3;
  }
  .connection-notice { background: #edf3fa; color: #35475d; }
  .error-notice { background: #fff0ef; color: #a52b23; }

  .status-dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: #8e8e93;
    transition: background 0.3s;
  }
  .status-dot.connected {
    background: var(--apple-green);
    box-shadow: 0 0 10px rgba(52, 199, 89, 0.7);
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    border: 0;
    padding: 0;
    background: transparent;
    text-align: left;
  }

  .device-name {
    margin: 0;
    font-size: 17px;
    font-weight: 600;
    letter-spacing: -0.3px;
    color: var(--lg-text-primary);
  }

  .edit-icon-svg {
    font-size: 11px;
    opacity: 0.5;
    transition: opacity 0.2s;
  }
  .title-row:hover .edit-icon-svg {
    opacity: 1;
  }

  .rename-inline {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .rename-input {
    background: rgba(255, 255, 255, 0.3);
    border: none;
    border-radius: 8px;
    padding: 3px 8px;
    font-size: 15px;
    font-weight: 500;
    color: var(--lg-text-primary);
    outline: none;
    width: 180px;
  }

  .save-name-btn {
    background: var(--apple-green);
    border: none;
    border-radius: 6px;
    color: white;
    font-size: 12px;
    padding: 3px 8px;
    cursor: pointer;
  }

  .glass-icon-btn {
    background: rgba(0, 0, 0, 0.05);
    border: none;
    border-radius: 50%;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--lg-text-primary);
    cursor: pointer;
    transition: all 0.2s;
  }
  .glass-icon-btn:hover {
    background: rgba(0, 0, 0, 0.1);
    transform: scale(1.05);
  }

  /* CƯỚP TAI NGHE TỪ IPHONE BUTTON */
  .takeover-row {
    width: 100%;
  }

  .takeover-btn {
    width: 100%;
    height: 38px;
    background: linear-gradient(135deg, rgba(0, 122, 255, 0.18), rgba(0, 122, 255, 0.08));
    border: none;
    border-radius: 14px;
    color: #007aff;
    font-size: 12.5px;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .takeover-btn:hover {
    background: linear-gradient(135deg, rgba(0, 122, 255, 0.28), rgba(0, 122, 255, 0.15));
    transform: scale(0.99);
  }
  .takeover-btn:active {
    transform: scale(0.97);
  }
  .takeover-btn.loading {
    opacity: 0.7;
    pointer-events: none;
  }

  /* Pin Strip */
  .battery-strip {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
  }

  .battery-tile {
    background: rgba(0, 0, 0, 0.04);
    border: none;
    border-radius: 16px;
    padding: 8px 6px;
    text-align: center;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .tile-label {
    font-size: 10px;
    color: var(--lg-text-secondary);
  }

  .tile-value {
    font-size: 15px;
    font-weight: 600;
    color: var(--lg-text-primary);
  }

  .tile-sub {
    font-size: 9.5px;
    color: var(--lg-text-secondary);
  }
  .tile-sub.active {
    color: var(--apple-green);
    font-weight: 500;
  }

  /* Section Blocks */
  .section-block {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .section-title {
    font-size: 10px;
    font-weight: 700;
    color: var(--lg-text-secondary);
    letter-spacing: 0.6px;
  }

  /* Pill Segmented - KHÔNG BORDER THÔ CỨNG */
  .pill-segmented {
    display: flex;
    background: rgba(0, 0, 0, 0.06);
    border: none;
    border-radius: 14px;
    padding: 3px;
    gap: 2px;
  }

  .pill-opt {
    flex: 1;
    background: transparent;
    border: none;
    border-radius: 11px;
    color: var(--lg-text-secondary);
    font-size: 12px;
    font-weight: 500;
    padding: 7px 2px;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .pill-opt.selected {
    background: rgba(255, 255, 255, 0.95);
    color: #111113;
    font-weight: 600;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
  }

  /* THANH TRƯỢT ADAPTIVE AUDIO */
  .adaptive-slider-card {
    background: rgba(0, 0, 0, 0.04);
    border-radius: 14px;
    padding: 8px 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 4px;
    animation: fade-down 0.25s ease;
  }

  @keyframes fade-down {
    from { opacity: 0; transform: translateY(-6px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .slider-labels {
    display: flex;
    justify-content: space-between;
    font-size: 9.5px;
    color: var(--lg-text-secondary);
  }

  .blend-level {
    font-weight: 600;
    color: var(--lg-text-primary);
  }

  .slider-track-wrap {
    width: 100%;
    display: flex;
    align-items: center;
  }

  .apple-range {
    width: 100%;
    height: 6px;
    -webkit-appearance: none;
    appearance: none;
    background: rgba(0, 0, 0, 0.12);
    border-radius: 4px;
    outline: none;
  }

  .apple-range::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #ffffff;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
    cursor: pointer;
    transition: transform 0.1s;
  }

  .apple-range::-webkit-slider-thumb:hover {
    transform: scale(1.15);
  }

  /* Seamless Settings List */
  .seamless-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: rgba(0, 0, 0, 0.04);
    border-radius: 18px;
    padding: 10px 14px;
  }

  .list-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .item-name {
    font-size: 12px;
    font-weight: 500;
    color: var(--lg-text-primary);
  }

  .item-desc {
    font-size: 10px;
    color: var(--lg-text-secondary);
  }

  /* Apple iOS Switch */
  .apple-switch {
    width: 38px;
    height: 22px;
    border-radius: 12px;
    background: rgba(0, 0, 0, 0.16);
    padding: 2px;
    box-sizing: border-box;
    cursor: pointer;
    transition: background 0.25s;
    flex-shrink: 0;
    border: 0;
  }
  .apple-switch.active {
    background: var(--apple-green);
  }
  .apple-switch:disabled {
    opacity: .45;
    cursor: not-allowed;
  }

  .switch-knob {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #ffffff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.25);
    transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .apple-switch.active .switch-knob {
    transform: translateX(16px);
  }

  .control-footer {
    margin-top: auto;
    text-align: center;
    font-size: 9.5px;
    color: var(--lg-text-secondary);
  }
</style>
