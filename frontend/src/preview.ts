import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

type Battery = { level: number | null; is_charging: boolean };
type CycleModes = { anc: boolean; adaptive: boolean; transparency: boolean; off: boolean };
type DeviceState = {
  name: string;
  address?: string;
  connected: boolean;
  model: unknown;
  noise_mode: string;
  battery: { left: Battery; right: Battery; case: Battery };
  in_ear?: { left_in_ear: boolean; right_in_ear: boolean };
  features: {
    conversational_awareness: boolean;
    personalized_volume?: boolean;
    loud_noise_reduction?: boolean;
    adaptive_audio_blend: number;
    keep_alive_connection?: boolean;
    auto_takeover?: boolean;
    spatial_audio_mode: string;
    stem_press_left?: string;
    stem_press_right?: string;
    cycle_modes?: CycleModes;
    one_bud_anc?: boolean;
    head_gestures?: boolean;
    auto_ear_detection?: boolean;
    case_charging_sounds?: boolean;
    case_chime_volume?: number;
    find_my_enabled?: boolean;
    spatial_calibrated?: boolean;
  };
};
type WindowsConnection = { connected: boolean; name: string | null };
type AudioOutput = { id: string; name: string; is_default: boolean };
type CurrentMedia = { title: string; artist: string; playing: boolean };

const popupView = location.hash === '#popup';
const settingsView = location.hash === '#settings';
const $ = (selector: string) => document.querySelector<HTMLElement>(selector);
const $$ = (selector: string) => [...document.querySelectorAll<HTMLElement>(selector)];

const HEADPHONE_SVG = `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 18v-6a9 9 0 0 1 18 0v6"/><path d="M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3zM3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3z"/></svg>`;
const SPEAKER_SVG = `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="13" rx="2"/><path d="M8 21h8"/><path d="M12 17v4"/></svg>`;
const MONITOR_SVG = `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>`;

const STEM_ACTIONS: Array<{ code: string; label: string }> = [
  { code: 'cyclenoisemodes', label: 'Chế độ nghe' },
  { code: 'siriorassistant', label: 'Trợ lý giọng nói' },
  { code: 'playpause', label: 'Phát / Tạm dừng' },
];

const deviceNameNodes = $$('span, div').filter(
  (element) => element.children.length === 0 && element.textContent?.trim() === 'Airpod 4 ANC của KXT',
);
const settingsOpenBtnSpan = $$('#mac-control-panel button span').find((el) =>
  el.textContent?.includes('Cài đặt Airpod'),
);

let currentState: DeviceState | null = null;
let windowsConnection: WindowsConnection = { connected: false, name: 'Airpod 4 ANC của KXT' };
let hasActiveAirPodsOutput: boolean | null = null;
let banner: HTMLDivElement | null = null;
let bannerTimer: ReturnType<typeof setTimeout> | null = null;

function isAirPodsAvailableOnWindows(): boolean {
  if (hasActiveAirPodsOutput === false) return false;
  return Boolean(windowsConnection.connected || currentState?.connected);
}

function syncAirPodsControlsEnabled() {
  const enabled = isAirPodsAvailableOnWindows();
  const controlSections = [
    $('#m-off')?.closest<HTMLElement>('#mac-control-panel > div'),
    $('#sp-off')?.closest<HTMLElement>('#mac-control-panel > div'),
    $('#conv-off')?.closest<HTMLElement>('#mac-control-panel > div'),
  ].filter((el): el is HTMLElement => Boolean(el));

  controlSections.forEach((section, idx) => {
    section.classList.toggle('airpods-control-disabled', !enabled);
    section.querySelectorAll<HTMLButtonElement | HTMLInputElement>('button, input').forEach((el) => {
      el.disabled = !enabled;
    });
    if (idx === 0) {
      const label = section.querySelector<HTMLElement>('.section-label');
      if (label) {
        let badge = label.querySelector<HTMLElement>('.airpods-offline-tag');
        if (!enabled) {
          if (!badge) {
            badge = document.createElement('span');
            badge.className = 'airpods-offline-tag';
            badge.textContent = 'Chưa kết nối';
            label.append(badge);
          }
        } else if (badge) {
          badge.remove();
        }
      }
    }
  });

  // Vô hiệu hóa các tùy chỉnh phần cứng AirPods trong Settings khi chưa kết nối
  $$('#settings-window-panel .settings-group-card').forEach((card) => {
    const text = card.textContent ?? '';
    if (text.includes('Ưu tiên cướp tai nghe')) return;
    card.classList.toggle('airpods-control-disabled', !enabled);
    card.querySelectorAll<HTMLButtonElement | HTMLInputElement>('button, input').forEach((el) => {
      el.disabled = !enabled;
    });
  });
}

// Preserve last known battery levels (fallback to preview defaults when case lid is closed)
let lastLeft: Battery = { level: 100, is_charging: false };
let lastRight: Battery = { level: 83, is_charging: false };
let lastCase: Battery = { level: 62, is_charging: false };

function showMessage(message: string, error = false) {
  if (popupView) return;
  if (!banner) {
    banner = document.createElement('div');
    banner.className = 'app-state-banner';
    document.body.append(banner);
  }
  banner.textContent = message;
  banner.classList.toggle('error', error);
  banner.style.display = 'block';
  if (bannerTimer) clearTimeout(bannerTimer);
  bannerTimer = setTimeout(() => {
    if (banner) banner.style.display = 'none';
  }, 4000);
}

function cleanOutputDisplayName(rawName: string): string {
  let s = rawName.trim();
  const matchParen = s.match(/^(?:Headphones|Headset)\s*\((.+)\)$/i);
  if (matchParen) {
    s = matchParen[1].trim();
  }
  s = s.replace(/\s*-\s*Find My(?:\s*Hands-Free)?$/i, '').trim();
  return s || rawName;
}

function getEffectiveBatteries(state?: DeviceState | null) {
  if (state?.battery) {
    if (state.battery.left.level != null) {
      lastLeft = state.battery.left;
    }
    if (state.battery.right.level != null) {
      lastRight = state.battery.right;
    }
    if (state.battery.case.level != null) {
      lastCase = state.battery.case;
    }
  }
  return { left: lastLeft, right: lastRight, case: lastCase };
}

function formatBatterySubtitle(state?: DeviceState | null): string {
  const b = getEffectiveBatteries(state);
  return `Tai L ${b.left.level ?? 100}% • Tai R ${b.right.level ?? 83}% • Hộp ${b.case.level ?? 62}%`;
}

function setRing(ring: Element | undefined, battery: Battery, circumference: number) {
  if (!ring) return;
  const lvl = battery.level ?? 0;
  const progress = ring.querySelector<HTMLElement>('.ring-progress-sm, .ring-progress');
  if (progress) {
    progress.style.strokeDashoffset = `${circumference * (1 - lvl / 100)}`;
  }
  const bolt = ring.querySelector<HTMLElement>('.ring-bolt-sm');
  if (bolt) {
    bolt.style.display = battery.is_charging ? '' : 'none';
  }
}

function updateBattery(state: DeviceState) {
  const eff = getEffectiveBatteries(state);
  const batteries = [eff.left, eff.right, eff.case];

  // 1. Control Center 3 compact battery rings
  const rings = $$('.top-split-card:first-child .apple-ring-container-sm');
  rings.forEach((ring, index) => {
    const battery = batteries[index];
    setRing(ring, battery, 94.2);
    const label = ring.parentElement?.querySelector<HTMLElement>('span');
    if (label) {
      const prefix = index === 0 ? 'L ' : index === 1 ? 'R ' : '';
      label.textContent = `${prefix}${battery.level ?? 0}%`;
    }
  });
  const count = $('.top-split-card:first-child .text-emerald-400');
  if (count) count.textContent = `● 3/3`;

  // 2. Settings Window 2 battery rings (Buds + Case)
  const settingsRings = $$('#settings-window-panel .apple-ring-container');
  [eff.left, eff.case].forEach((battery, index) => {
    const ring = settingsRings[index];
    setRing(ring, battery, 138.2);
    const label = ring?.querySelector<HTMLElement>('span');
    if (label) label.textContent = `${battery.level ?? 0}%`;
  });

  // 3. Bottom-Right Connection Popup
  const popupBuds = $$('#bottom-right-container .flex.flex-col.items-center.w-16');
  [eff.left, eff.right].forEach((battery, index) => {
    const bud = popupBuds[index];
    const label = bud?.lastElementChild as HTMLElement | null;
    if (label) label.textContent = `${battery.level ?? 0}%`;
    const fill = bud?.querySelectorAll<SVGRectElement>('svg rect')[1];
    if (fill) fill.setAttribute('width', `${(17 * (battery.level ?? 0)) / 100}`);
    const bolt = bud?.querySelectorAll<HTMLElement>('svg')[1];
    if (bolt) bolt.style.display = battery.is_charging ? '' : 'none';
  });
  const popupCase = $('#bottom-right-container .flex.flex-col.items-center.w-32');
  const caseLabel = popupCase?.lastElementChild as HTMLElement | null;
  if (caseLabel) caseLabel.textContent = `${eff.case.level ?? 0}%`;
  const caseFill = popupCase?.querySelectorAll<SVGRectElement>('svg rect')[1];
  if (caseFill) caseFill.setAttribute('width', `${(17 * (eff.case.level ?? 0)) / 100}`);

  // 4. Audio Output battery subtitle
  $$('[data-airpods-battery-sub]').forEach((el) => {
    el.textContent = formatBatterySubtitle(state);
  });
}

function applyNoiseModeUI(uiKey: 'off' | 'trans' | 'adapt' | 'anc') {
  const keys: Array<'off' | 'trans' | 'adapt' | 'anc'> = ['off', 'trans', 'adapt', 'anc'];
  for (const k of keys) {
    $$(`[onclick="setNoiseMode('${k}')"]`).forEach((btn) => {
      btn.classList.toggle('active', k === uiKey);
    });
  }
  const adaptiveBox = $('#adaptive-slider-box');
  if (adaptiveBox) {
    adaptiveBox.style.display = uiKey === 'adapt' ? 'block' : 'none';
  }
}

function applySpatialModeUI(uiKey: 'off' | 'fixed' | 'head') {
  const keys: Array<'off' | 'fixed' | 'head'> = ['off', 'fixed', 'head'];
  for (const k of keys) {
    $$(`[onclick="setSpatialMode('${k}')"]`).forEach((btn) => {
      btn.classList.toggle('active', k === uiKey);
    });
  }
}

function applyConversationModeUI(enabled: boolean) {
  $('#conv-off')?.classList.toggle('active', !enabled);
  $('#conv-on')?.classList.toggle('active', enabled);
  $$('#settings-window-panel .settings-row').forEach((row) => {
    const title = row.querySelector('span')?.textContent?.trim() ?? '';
    if (title.includes('Nhận biết cuộc hội thoại')) {
      const cb = row.querySelector<HTMLInputElement>('input[type="checkbox"]');
      if (cb) cb.checked = enabled;
    }
  });
}

function applyAdaptiveBlendUI(blend: number) {
  const slider = $('#adapt-slider-input') as HTMLInputElement | null;
  if (slider) slider.value = String(blend);
  const readout = $('#adapt-readout');
  if (readout) {
    readout.textContent = blend === 50 ? '50% (Cân bằng)' : `${blend}%`;
  }
}

function stemLabelFromCode(code?: string): string {
  const found = STEM_ACTIONS.find((a) => a.code === (code ?? '').toLowerCase());
  return found ? found.label : 'Chế độ nghe';
}

function syncSettingsPanelFromState(state: DeviceState) {
  const f = state.features;
  $$('#settings-window-panel .settings-row').forEach((row) => {
    const title = row.querySelector('span')?.textContent?.trim() ?? '';
    const cb = row.querySelector<HTMLInputElement>('input[type="checkbox"]');
    if (cb) {
      if (title.includes('Kiểm soát tiếng ồn bằng một AirPod') && f.one_bud_anc !== undefined) {
        cb.checked = f.one_bud_anc;
      } else if (title.includes('Nhận biết cuộc hội thoại')) {
        cb.checked = Boolean(f.conversational_awareness);
      } else if (title.includes('Âm lượng được cá nhân hóa') && f.personalized_volume !== undefined) {
        cb.checked = f.personalized_volume;
      } else if (title.includes('Giảm âm thanh lớn') && f.loud_noise_reduction !== undefined) {
        cb.checked = f.loud_noise_reduction;
      } else if (title.includes('Tương tác bằng cử chỉ đầu') && f.head_gestures !== undefined) {
        cb.checked = f.head_gestures;
      } else if (title.includes('Tự động phát hiện tai') && f.auto_ear_detection !== undefined) {
        cb.checked = f.auto_ear_detection;
      } else if (title.includes('Ưu tiên cướp tai nghe') && f.auto_takeover !== undefined) {
        cb.checked = f.auto_takeover;
      } else if (title.includes('Luôn duy trì kết nối L2CAP') && f.keep_alive_connection !== undefined) {
        cb.checked = f.keep_alive_connection;
      } else if (title.includes('Bật âm thanh hộp sạc') && f.case_charging_sounds !== undefined) {
        cb.checked = f.case_charging_sounds;
      }
    }

    if (title === 'Bên trái') {
      const valueSpan = row.querySelector('div span');
      if (valueSpan) valueSpan.textContent = stemLabelFromCode(f.stem_press_left);
    } else if (title === 'Bên phải') {
      const valueSpan = row.querySelector('div span');
      if (valueSpan) valueSpan.textContent = stemLabelFromCode(f.stem_press_right);
    }
  });

  if (f.cycle_modes) {
    const cycleBoxes = $$('#settings-window-panel .flex-wrap label input[type="checkbox"]') as HTMLInputElement[];
    if (cycleBoxes.length === 4) {
      cycleBoxes[0].checked = f.cycle_modes.anc;
      cycleBoxes[1].checked = f.cycle_modes.adaptive;
      cycleBoxes[2].checked = f.cycle_modes.transparency;
      cycleBoxes[3].checked = f.cycle_modes.off;
    }
  }

  if (f.case_chime_volume !== undefined) {
    const chimeVal = $('#chime-val');
    if (chimeVal) chimeVal.textContent = `${f.case_chime_volume}%`;
    const chimeSlider = $('#chime-val')?.closest('.settings-group-card')?.querySelector<HTMLInputElement>('input[type="range"]');
    if (chimeSlider) chimeSlider.value = String(f.case_chime_volume);
  }
}

function updateSelections(state: DeviceState) {
  const noiseMap: Record<string, 'off' | 'trans' | 'adapt' | 'anc'> = {
    off: 'off',
    transparency: 'trans',
    adaptive: 'adapt',
    noisecancellation: 'anc',
  };
  applyNoiseModeUI(noiseMap[state.noise_mode] || 'anc');

  const spatialMap: Record<string, 'off' | 'fixed' | 'head'> = {
    off: 'off',
    fixed: 'fixed',
    headtracked: 'head',
  };
  applySpatialModeUI(spatialMap[state.features.spatial_audio_mode] || 'fixed');

  applyConversationModeUI(Boolean(state.features.conversational_awareness));
  applyAdaptiveBlendUI(state.features.adaptive_audio_blend ?? 50);
  syncSettingsPanelFromState(state);
}

function updateDevice(state: DeviceState) {
  currentState = state;
  updateBattery(state);
  updateSelections(state);

  const displayName = state.name || windowsConnection.name || 'Airpod 4 ANC của KXT';
  deviceNameNodes.forEach((node) => {
    node.textContent = displayName;
  });

  if (settingsOpenBtnSpan) {
    const svg = settingsOpenBtnSpan.querySelector('svg');
    settingsOpenBtnSpan.textContent = ` Cài đặt ${displayName}...`;
    if (svg) settingsOpenBtnSpan.prepend(svg);
  }

  const nameInput = $('#settings-window-panel input[type="text"]') as HTMLInputElement | null;
  if (nameInput && document.activeElement !== nameInput) {
    nameInput.value = displayName;
  }
  syncAirPodsControlsEnabled();
}

function updateWindowsConnection(connection: WindowsConnection) {
  windowsConnection = connection;
  if (currentState) updateDevice(currentState);
  syncAirPodsControlsEnabled();
  if (!popupView) void refreshAudioOutputs();
}

let takeoverInFlight = false;

async function refreshAudioOutputs(force = false) {
  if (takeoverInFlight && !force) return;
  const list = $('#mac-control-panel .output-list-card');
  if (!list) return;
  try {
    const outputs = await invoke<AudioOutput[]>('get_audio_outputs');
    if (!outputs || outputs.length === 0) return;

    const knownName = (currentState?.name || windowsConnection.name || '').toLowerCase();
    const isAirPodsName = (name: string) => {
      const lower = name.toLowerCase();
      return lower.includes('airpod') || (knownName.length > 0 && lower.includes(knownName));
    };

    // Filter out duplicate mono Hands-Free Headset entry if stereo AirPods Headphones exists
    const hasStereoAirPods = outputs.some(
      (o) => isAirPodsName(o.name) && o.name.toLowerCase().includes('headphones'),
    );
    const filteredOutputs = outputs.filter((o) => {
      const lower = o.name.toLowerCase();
      if (hasStereoAirPods && isAirPodsName(o.name) && lower.includes('headset')) {
        return false;
      }
      return true;
    });

    list.replaceChildren();

    // Nếu AirPods đang kết nối iPhone (chưa có trong danh sách Active của Windows),
    // vẫn hiển thị hàng AirPods ở trên cùng để người dùng bấm 1 chạm cướp kết nối về PC!
    const hasAnyAirPodsActive = filteredOutputs.some((o) => isAirPodsName(o.name));
    hasActiveAirPodsOutput = hasAnyAirPodsActive;
    syncAirPodsControlsEnabled();
    if (!hasAnyAirPodsActive) {
      const takeoverRow = document.createElement('div');
      takeoverRow.className = 'output-item-row';
      const leftGroup = document.createElement('div');
      leftGroup.className = 'flex items-center gap-2.5 overflow-hidden';
      const iconCircle = document.createElement('div');
      iconCircle.className =
        'w-6 h-6 rounded-full bg-blue-500/20 text-blue-400 flex items-center justify-center shrink-0';
      iconCircle.innerHTML = HEADPHONE_SVG;
      const textCol = document.createElement('div');
      textCol.className = 'flex flex-col overflow-hidden';
      const titleSpan = document.createElement('span');
      titleSpan.className = 'device-name-clamp font-medium text-white';
      titleSpan.textContent = currentState?.name || windowsConnection.name || 'AirPods 4 ANC';
      const subSpan = document.createElement('span');
      subSpan.className = 'text-[10px] text-blue-300/80';
      subSpan.textContent = 'Đang ở iPhone / Ngắt kết nối • Bấm để cướp về PC';
      textCol.append(titleSpan, subSpan);
      leftGroup.append(iconCircle, textCol);
      const badgeSpan = document.createElement('span');
      badgeSpan.className = 'text-[10px] font-semibold text-blue-400 shrink-0';
      badgeSpan.textContent = 'Kết nối';
      takeoverRow.append(leftGroup, badgeSpan);
      takeoverRow.addEventListener('click', async () => {
        if (takeoverInFlight) return;
        takeoverInFlight = true;
        badgeSpan.textContent = 'Đang kéo...';
        try {
          await invoke('force_takeover_audio');
          showMessage('Đã cướp kết nối AirPods từ iPhone sang PC thành công ✓', false);
          await refreshAudioOutputs(true);
          await refreshState();
          await refreshVolume();
        } catch (error) {
          badgeSpan.textContent = 'Kết nối';
          showMessage(String(error), true);
        } finally {
          takeoverInFlight = false;
        }
      });
      list.append(takeoverRow);
    }

    for (const output of filteredOutputs) {
      const lower = output.name.toLowerCase();
      const isAirPods = isAirPodsName(output.name);
      const isMonitor =
        lower.includes('hdmi') ||
        lower.includes('display') ||
        lower.includes('monitor') ||
        lower.includes('dell');

      const row = document.createElement('div');
      row.className = `output-item-row${output.is_default ? ' active' : ''}`;

      const leftGroup = document.createElement('div');
      leftGroup.className = 'flex items-center gap-2.5 overflow-hidden';

      const iconCircle = document.createElement('div');
      iconCircle.className = isAirPods
        ? 'w-6 h-6 rounded-full bg-blue-500/20 text-blue-400 flex items-center justify-center shrink-0'
        : 'w-6 h-6 rounded-full bg-white/10 text-white/70 flex items-center justify-center shrink-0';
      iconCircle.innerHTML = isAirPods ? HEADPHONE_SVG : isMonitor ? MONITOR_SVG : SPEAKER_SVG;

      if (isAirPods) {
        const textCol = document.createElement('div');
        textCol.className = 'flex flex-col overflow-hidden';

        const titleSpan = document.createElement('span');
        titleSpan.className = 'device-name-clamp font-medium text-white';
        titleSpan.textContent = currentState?.name || cleanOutputDisplayName(output.name);

        const subSpan = document.createElement('span');
        subSpan.className = 'text-[10px] text-white/50';
        subSpan.setAttribute('data-airpods-battery-sub', 'true');
        subSpan.textContent = formatBatterySubtitle(currentState);

        textCol.append(titleSpan, subSpan);
        leftGroup.append(iconCircle, textCol);
      } else {
        const titleSpan = document.createElement('span');
        titleSpan.className = `device-name-clamp ${output.is_default ? 'font-medium text-white' : 'text-white/80'}`;
        titleSpan.textContent = output.name;
        leftGroup.append(iconCircle, titleSpan);
      }

      const checkSpan = document.createElement('span');
      checkSpan.className = 'font-bold text-white shrink-0';
      checkSpan.textContent = output.is_default ? '✓' : '';

      row.append(leftGroup, checkSpan);
      row.addEventListener('click', async () => {
        if (takeoverInFlight) return;
        $$('#mac-control-panel .output-item-row').forEach((r) => {
          r.classList.remove('active');
          const chk = r.lastElementChild;
          if (chk) chk.textContent = '';
        });
        row.classList.add('active');
        checkSpan.textContent = isAirPods ? '...' : '✓';
        if (isAirPods) takeoverInFlight = true;
        try {
          if (isAirPods) {
            await invoke('force_takeover_audio');
          } else {
            await invoke('set_default_audio_output', { id: output.id });
          }
          await refreshAudioOutputs(true);
          await refreshVolume();
        } catch (error) {
          showMessage(String(error), true);
          await refreshAudioOutputs(true);
        } finally {
          takeoverInFlight = false;
        }
      });

      list.append(row);
    }
  } catch {
    // Keep existing static preview rows if Windows audio enumeration fails
  }
}

async function refreshState() {
  try {
    const state = await invoke<DeviceState>('get_device_state');
    updateDevice(state);
  } catch {
    // Keep preview defaults
  }
}

async function sendCommand(command: string, args: Record<string, unknown>) {
  try {
    await invoke(command, args);
    if (banner) banner.style.display = 'none';
    await refreshState();
  } catch (error) {
    showMessage(String(error), true);
  }
}

function switchMainView(view: string) {
  void invoke(view === 'settings' ? 'show_settings_window' : 'hide_settings_window').catch(
    (error) => showMessage(String(error), true),
  );
}

async function triggerConnectionPopup() {
  if (popupView) return;
  try {
    await invoke('show_connection_popup');
  } catch (error) {
    showMessage(String(error), true);
  }
}

function resetPopupAnimation() {
  const popup = $('#bottom-right-container');
  if (!popup) return;
  popup.classList.remove('popup-spring-out');
  popup.classList.remove('popup-spring-in');
  void popup.offsetWidth;
  popup.classList.add('popup-spring-in');
}

function closeConnectionPopup() {
  const popup = $('#bottom-right-container');
  popup?.classList.remove('popup-spring-in');
  popup?.classList.add('popup-spring-out');
  setTimeout(() => {
    void invoke('close_popup').catch(() => {});
  }, 160);
}

Object.assign(window, {
  switchMainView,
  openSettingsModal: () => switchMainView('settings'),
  closeSettingsModal: () => switchMainView('control'),
  triggerConnectionPopup,
  closeConnectionPopup,
  setNoiseMode: (mode: 'off' | 'trans' | 'adapt' | 'anc') => {
    if (!isAirPodsAvailableOnWindows()) return;
    applyNoiseModeUI(mode);
    const values: Record<string, number> = { off: 1, anc: 2, trans: 3, adapt: 4 };
    void sendCommand('set_noise_mode', { mode: values[mode] });
  },
  setSpatialMode: (mode: 'off' | 'fixed' | 'head') => {
    if (!isAirPodsAvailableOnWindows()) return;
    applySpatialModeUI(mode);
    const values: Record<string, string> = { off: 'off', fixed: 'fixed', head: 'headtracked' };
    void sendCommand('set_spatial_audio_mode', { mode: values[mode] });
  },
  setConversationMode: (enabled: boolean) => {
    if (!isAirPodsAvailableOnWindows()) return;
    applyConversationModeUI(enabled);
    void sendCommand('toggle_conversational_awareness', { enabled });
  },
  updateAdaptiveSlider: (value: string) => {
    if (!isAirPodsAvailableOnWindows()) return;
    const num = Number(value);
    applyAdaptiveBlendUI(num);
    void sendCommand('set_adaptive_blend', { level: num });
  },
  resetAdaptiveBalance: () => {
    if (!isAirPodsAvailableOnWindows()) return;
    applyAdaptiveBlendUI(50);
    void sendCommand('set_adaptive_blend', { level: 50 });
  },
  selectWindowsOutput: (target: string) => {
    const outs = ['airpods', 'speakers', 'hdmi'];
    for (const o of outs) {
      const row = document.getElementById('out-' + o);
      const chk = document.getElementById('chk-out-' + o);
      if (row) row.classList.toggle('active', o === target);
      if (chk) chk.innerText = o === target ? '✓' : '';
    }
  },
  toggleMediaPlayback: () => void sendMediaCommand('toggle'),
});

async function sendMediaCommand(action: 'previous' | 'toggle' | 'next') {
  if (action === 'toggle') {
    const pauseIcon = $('#icon-pause');
    const playIcon = $('#icon-play');
    const isCurrentlyPlaying = pauseIcon ? pauseIcon.style.display !== 'none' : true;
    if (pauseIcon) pauseIcon.style.display = isCurrentlyPlaying ? 'none' : 'block';
    if (playIcon) playIcon.style.display = isCurrentlyPlaying ? 'block' : 'none';
  }
  try {
    await invoke('media_control', { action });
    setTimeout(() => {
      void refreshMedia();
    }, 300);
  } catch {
    // Ignore when no external media player is open
  }
}

async function refreshMedia() {
  try {
    const media = await invoke<CurrentMedia | null>('get_current_media');
    if (!media || !media.title) return;

    const track = $('.album-art-box-sm + div');
    if (track) {
      track.replaceChildren();
      const title = document.createElement('span');
      title.className = 'text-[11.5px] font-semibold text-white truncate';
      title.textContent = media.title;
      const artist = document.createElement('span');
      artist.className = 'text-[9.5px] text-white/50 truncate';
      artist.textContent = media.artist || 'Windows Media';
      track.append(title, artist);
    }
    const pauseIcon = $('#icon-pause');
    const playIcon = $('#icon-play');
    if (pauseIcon) pauseIcon.style.display = media.playing ? 'block' : 'none';
    if (playIcon) playIcon.style.display = media.playing ? 'none' : 'block';
  } catch {
    // Preserve default preview Now Playing card
  }
}

async function refreshVolume() {
  try {
    const level = await invoke<number>('get_windows_volume');
    const label = $('#vol-pct');
    const fill = $('#vol-fill');
    if (label) label.textContent = `${level}%`;
    if (fill) fill.style.width = `${level}%`;
  } catch {
    // Keep preview volume
  }
}

function setupVolumeAndTransport() {
  // Replace alert('Bài trước') / alert('Bài tiếp theo') with real Windows media skip
  $$('.media-btn-sm').forEach((button, index) => {
    button.removeAttribute('onclick');
    button.addEventListener('click', () => {
      void sendMediaCommand(index === 0 ? 'previous' : 'next');
    });
  });

  const track = $('#vol-track');
  if (!track) return;
  let dragging = false;
  const setFromPointer = (event: PointerEvent) => {
    const rect = track.getBoundingClientRect();
    const level = Math.round(
      Math.max(0, Math.min(1, (event.clientX - rect.left) / rect.width)) * 100,
    );
    const label = $('#vol-pct');
    const fill = $('#vol-fill');
    if (label) label.textContent = `${level}%`;
    if (fill) fill.style.width = `${level}%`;
    return level;
  };
  track.addEventListener('pointerdown', (event) => {
    dragging = true;
    track.setPointerCapture(event.pointerId);
    const level = setFromPointer(event);
    void invoke('set_windows_volume', { level }).catch(() => {});
  });
  track.addEventListener('pointermove', (event) => {
    if (dragging) setFromPointer(event);
  });
  track.addEventListener('pointerup', (event) => {
    if (!dragging) return;
    dragging = false;
    const level = setFromPointer(event);
    void invoke('set_windows_volume', { level }).catch(() => {});
  });
  // Support mouse wheel scrolling on volume card
  const volCard = track.closest('.volume-capsule-card') ?? track;
  volCard.addEventListener(
    'wheel',
    (event) => {
      event.preventDefault();
      const we = event as WheelEvent;
      const label = $('#vol-pct');
      const current = parseInt(label?.textContent ?? '50', 10) || 50;
      const delta = we.deltaY < 0 ? 4 : -4;
      const next = Math.max(0, Math.min(100, current + delta));
      if (label) label.textContent = `${next}%`;
      const fill = $('#vol-fill');
      if (fill) fill.style.width = `${next}%`;
      void invoke('set_windows_volume', { level: next }).catch(() => {});
    },
    { passive: false },
  );
  void refreshVolume();
}

function setupSettingsInteractions() {
  // 1. Device Rename Input
  const nameInput = $('#settings-window-panel input[type="text"]') as HTMLInputElement | null;
  const saveRename = () => {
    if (!nameInput) return;
    const newName = nameInput.value.trim();
    if (newName) void sendCommand('rename_device', { newName });
  };
  nameInput?.addEventListener('change', saveRename);
  nameInput?.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      nameInput.blur();
      saveRename();
    }
  });

  // 2. Settings rows (switches, stem actions, acoustic fit test)
  $$('#settings-window-panel .settings-row').forEach((row) => {
    const title = row.querySelector('span')?.textContent?.trim() ?? '';
    const checkbox = row.querySelector<HTMLInputElement>('input[type="checkbox"]');

    if (checkbox) {
      if (title.includes('Kiểm soát tiếng ồn bằng một AirPod')) {
        checkbox.addEventListener('change', () => {
          void sendCommand('set_one_bud_anc', { enabled: checkbox.checked });
        });
      } else if (title.includes('Nhận biết cuộc hội thoại')) {
        checkbox.addEventListener('change', () => {
          applyConversationModeUI(checkbox.checked);
          void sendCommand('toggle_conversational_awareness', { enabled: checkbox.checked });
        });
      } else if (title.includes('Âm lượng được cá nhân hóa')) {
        checkbox.addEventListener('change', () => {
          void sendCommand('set_personalized_volume', { enabled: checkbox.checked });
        });
      } else if (title.includes('Giảm âm thanh lớn')) {
        checkbox.addEventListener('change', () => {
          void sendCommand('set_loud_noise_reduction', { enabled: checkbox.checked });
        });
      } else if (title.includes('Tương tác bằng cử chỉ đầu')) {
        checkbox.addEventListener('change', () => {
          void sendCommand('set_head_gestures', { enabled: checkbox.checked });
        });
      } else if (title.includes('Tự động phát hiện tai')) {
        checkbox.addEventListener('change', () => {
          void sendCommand('set_auto_ear_detection', { enabled: checkbox.checked });
        });
      } else if (title.includes('Ưu tiên cướp tai nghe')) {
        checkbox.addEventListener('change', () => {
          void sendCommand('set_auto_takeover', { enabled: checkbox.checked });
        });
      } else if (title.includes('Luôn duy trì kết nối L2CAP')) {
        checkbox.addEventListener('change', () => {
          void sendCommand('set_keep_alive', { enabled: checkbox.checked });
        });
      } else if (title.includes('Bật âm thanh hộp sạc')) {
        checkbox.addEventListener('change', () => {
          void sendCommand('set_case_charging_sounds', { enabled: checkbox.checked });
        });
      }
      return;
    }

    // Stem press action rows: Bên trái / Bên phải
    if (title === 'Bên trái' || title === 'Bên phải') {
      const side = title === 'Bên phải' ? 'right' : 'left';
      row.style.cursor = 'pointer';
      row.addEventListener('click', () => {
        const valueSpan = row.querySelector('div span');
        const currentLabel = valueSpan?.textContent?.trim() ?? 'Chế độ nghe';
        const idx = STEM_ACTIONS.findIndex((a) => a.label === currentLabel);
        const nextAction = STEM_ACTIONS[(idx + 1) % STEM_ACTIONS.length];
        if (valueSpan) valueSpan.textContent = nextAction.label;
        void sendCommand('set_stem_press_action', { side, action: nextAction.code });
      });
    }

    // Acoustic Fit Test button ("Kiểm tra độ vừa vặn âm học")
    if (title.includes('Kiểm tra độ vừa vặn âm học')) {
      const btn = row.querySelector<HTMLButtonElement>('button');
      btn?.addEventListener('click', async () => {
        btn.textContent = 'Đang kiểm tra...';
        btn.disabled = true;
        try {
          const msg = await invoke<string>('run_acoustic_fit_test');
          setTimeout(() => {
            btn.textContent = 'Đạt chuẩn ✓';
            btn.disabled = false;
            showMessage(msg, false);
          }, 600);
        } catch (error) {
          btn.textContent = 'Kiểm tra...';
          btn.disabled = false;
          showMessage(String(error), true);
        }
      });
    }
  });

  // 3. Cycle modes checkboxes ("Luân chuyển giữa các chế độ:")
  const cycleBoxes = $$('#settings-window-panel .flex-wrap label input[type="checkbox"]') as HTMLInputElement[];
  if (cycleBoxes.length === 4) {
    cycleBoxes.forEach((box) => {
      box.addEventListener('change', () => {
        const anc = cycleBoxes[0].checked;
        const adaptive = cycleBoxes[1].checked;
        const transparency = cycleBoxes[2].checked;
        const off = cycleBoxes[3].checked;
        const count = [anc, adaptive, transparency, off].filter(Boolean).length;
        if (count < 2) {
          box.checked = true;
          showMessage('Cần chọn ít nhất 2 chế độ nghe để luân chuyển khi nhấn giữ AirPods', true);
          return;
        }
        void sendCommand('set_cycle_modes', { anc, adaptive, transparency, off });
      });
    });
  }

  // 4. Case chime volume slider ("Âm lượng chuông hộp sạc:")
  const chimeSlider = $('#chime-val')?.closest('.settings-group-card')?.querySelector<HTMLInputElement>('input[type="range"]');
  chimeSlider?.addEventListener('change', () => {
    const volume = Number(chimeSlider.value);
    void sendCommand('set_case_chime_volume', { volume });
  });

  // 5. Forget / Disconnect button ("Quên thiết bị này")
  const forgetBtn = $$('#settings-window-panel button').find((b) =>
    b.textContent?.includes('Quên thiết bị này'),
  );
  forgetBtn?.addEventListener('click', async () => {
    try {
      await invoke('disconnect_device');
      showMessage('Đã ngắt phiên điều khiển AirPods. Sẽ tự động kết nối lại khi sử dụng.', false);
    } catch (error) {
      showMessage(String(error), true);
    }
  });
}

if (popupView) {
  document.body.classList.add('popup-view');
  resetPopupAnimation();
  void listen<number>('popup-shown', () => {
    resetPopupAnimation();
    void refreshState();
  });
} else {
  document.body.classList.add(settingsView ? 'settings-view' : 'control-view');
  setupVolumeAndTransport();
  setupSettingsInteractions();
  syncAirPodsControlsEnabled();
  if (!settingsView) {
    void refreshAudioOutputs();
    void refreshMedia();
    setInterval(() => {
      void refreshAudioOutputs();
      void refreshMedia();
      void refreshVolume();
    }, 3000);
  }
}

void refreshState();
window.addEventListener('focus', () => {
  void refreshState();
});
document.addEventListener('visibilitychange', () => {
  if (!document.hidden) void refreshState();
});
void listen<DeviceState>('device-state-updated', (event) => updateDevice(event.payload));
void invoke<WindowsConnection>('get_windows_connection')
  .then(updateWindowsConnection)
  .catch(() => {});
void listen<WindowsConnection>('windows-connection-changed', (event) =>
  updateWindowsConnection(event.payload),
);
void listen<boolean>('audio-outputs-changed', () => {
  if (!popupView && !settingsView) {
    void refreshAudioOutputs();
    void refreshVolume();
  }
});
void listen<boolean>('flyout-shown', () => {
  if (!popupView && !settingsView) {
    const panel = $('#mac-control-panel');
    if (panel) {
      panel.classList.remove('re-enter');
      void panel.offsetWidth;
      panel.classList.add('re-enter');
    }
    void refreshState();
    void refreshAudioOutputs();
    void refreshMedia();
    void refreshVolume();
  }
});
void listen<string>('control-error', (event) => showMessage(event.payload, true));
