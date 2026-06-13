import { initOverlay, renderFrame, setPaintColor } from './renderer.js';

const TAURI = window.__TAURI__;

let currentMode = 'menu'; // 'menu' | 'overlay' | 'debug'
let menuEntries = [];
let gamepadState = null;
let debugCardVisible = false;

document.addEventListener('DOMContentLoaded', async () => {
  // Disable right-click context menu
  document.addEventListener('contextmenu', e => e.preventDefault());

  if (!TAURI) {
    document.getElementById('menu-status').textContent = 'Tauri API not available';
    return;
  }

  // Close button
  document.getElementById('close-btn').addEventListener('click', async () => {
    try {
      await TAURI.window.getCurrentWindow().close();
    } catch (e) { /* ignore */ }
  });

  // Reveal debug menu card on ~ key (only at top-level menu)
  document.addEventListener('keydown', (e) => {
    if (e.key === '`' || e.key === '~') {
      if (currentMode !== 'menu' || menuStack.length > 0) return;
      debugCardVisible = !debugCardVisible;
      showMenu();
    }
  });

  // Auto-refresh menu when controllers directory changes
  if (TAURI.event) {
    let refreshTimer = null;
    TAURI.event.listen('controllers-changed', () => {
      if (currentMode !== 'menu') return;
      if (refreshTimer) clearTimeout(refreshTimer);
      refreshTimer = setTimeout(() => {
        refreshTimer = null;
        showMenu();
      }, 400);
    });
  }

  // Drag window on pointer-down (except on interactive elements)
  document.getElementById('menu-screen').addEventListener('pointerdown', onDragStart);
  document.getElementById('overlay-screen').addEventListener('pointerdown', onDragStart);
  document.getElementById('debug-screen').addEventListener('pointerdown', onDragStart);

  await showMenu();
});

function onDragStart(e) {
  let t = e.target;
  // Only drag from background/empty areas, not interactive elements
  if (t.closest('button') || t.closest('input') || t.closest('.menu-card') ||
      t.closest('#gear-btn') || t.closest('#settings-panel') ||
      t.closest('#close-btn') || t.closest('#back-btn')) return;
  if (!TAURI) return;
  TAURI.window.getCurrentWindow().startDragging();
}

// ─── Menu ───

let menuStack = []; // path stack for back navigation

async function showMenu() {
  currentMode = 'menu';
  document.getElementById('menu-screen').style.display = 'flex';
  document.getElementById('overlay-screen').style.display = 'none';

  menuStack = [];
  document.getElementById('back-btn').style.display = 'none';

  let groups;
  try {
    groups = await TAURI.core.invoke('list_groups');
  } catch (e) {
    document.getElementById('menu-status').textContent = 'Error: ' + e;
    return;
  }

  renderCards(groups, 'Select a Controller', false);
}

function renderCards(items, header, areControllers) {
  document.getElementById('menu-header').textContent = header;

  let grid = document.getElementById('menu-grid');
  grid.innerHTML = '';

  if (items.length === 0) {
    grid.innerHTML = '<div style="color:#888">No controllers found.</div>';
    document.getElementById('menu-status').textContent = 'No controllers detected';
    return;
  }

  for (let item of items) {
    let card = document.createElement('div');
    card.className = 'menu-card';

    if (item.icon_data_url) {
      let img = document.createElement('img');
      img.className = 'icon';
      img.src = item.icon_data_url;
      card.appendChild(img);
    } else {
      let letter = document.createElement('div');
      letter.style.cssText = 'width:140px;height:140px;display:flex;align-items:center;justify-content:center;font-size:56px;color:#aaa;';
      letter.textContent = item.name.charAt(0);
      card.appendChild(letter);
    }

    let name = document.createElement('div');
    name.className = 'name';
    name.textContent = item.name;
    card.appendChild(name);

    card.onclick = (item.is_controller ?? areControllers)
      ? () => selectController(item)
      : () => drillIntoGroup(item);
    grid.appendChild(card);
  }

  // Append debug card at end of top-level groups
  if (!areControllers && debugCardVisible) {
    let card = document.createElement('div');
    card.className = 'menu-card';
    card.innerHTML = '<div style="width:140px;height:140px;display:flex;align-items:center;justify-content:center;font-size:56px;color:#f80">🔧</div>' +
      '<div class="name" style="color:#f80">Controller Debug</div>';
    card.onclick = () => showDebug();
    grid.appendChild(card);
  }

  document.getElementById('menu-status').textContent = `${items.length} item(s)`;
}

async function drillIntoGroup(group) {
  debugCardVisible = false;
  document.getElementById('menu-status').textContent = 'Loading…';
  try {
    let controllers = await TAURI.core.invoke('list_group_controllers', { path: group.path });
    if (controllers.length === 0) {
      document.getElementById('menu-status').textContent = 'No controllers in this group';
      return;
    }
    menuStack.push(group);
    document.getElementById('back-btn').style.display = 'block';
    renderCards(controllers, group.name, true);
  } catch (e) {
    document.getElementById('menu-status').textContent = 'Error: ' + e;
  }
}

document.getElementById('back-btn').addEventListener('click', async () => {
  if (menuStack.length === 0) {
    await showMenu();
  } else {
    menuStack.pop();
    if (menuStack.length === 0) {
      document.getElementById('back-btn').style.display = 'none';
      await showMenu();
    } else {
      // Go back one more level
      let parent = menuStack[menuStack.length - 1];
      let controllers = await TAURI.core.invoke('list_group_controllers', { path: parent.path });
      renderCards(controllers, parent.name, true);
    }
  }
});

async function selectController(entry) {
  debugCardVisible = false;
  document.getElementById('menu-status').textContent = `Loading ${entry.name}...`;
  try {
    let result = await TAURI.core.invoke('load_controller', { path: entry.path });
    showOverlay(result.def, result.mode, entry.path, result.color);
  } catch (e) {
    document.getElementById('menu-status').textContent = 'Failed to load: ' + e;
  }
}

// ─── Debug ───

let debugPollId = null;
let selectedDeviceId = null;
let debugDeviceRefreshCounter = 0;

document.getElementById('debug-back-btn').addEventListener('click', async () => {
  closeDebug();
});

async function showDebug() {
  currentMode = 'debug';
  document.getElementById('menu-screen').style.display = 'none';
  document.getElementById('overlay-screen').style.display = 'none';
  document.getElementById('debug-screen').style.display = 'flex';

  selectedDeviceId = null;
  debugDeviceRefreshCounter = 0;
  await refreshDebugDeviceList();
  startDebugPoll();
}

function closeDebug() {
  currentMode = 'menu';
  debugCardVisible = false;
  if (debugPollId) { cancelAnimationFrame(debugPollId); debugPollId = null; }
  TAURI.core.invoke('close_debug_gamepad').catch(() => {});
  document.getElementById('debug-screen').style.display = 'none';
  document.getElementById('menu-screen').style.display = 'flex';
  showMenu();
}

function startDebugPoll() {
  function loop() {
    if (currentMode !== 'debug') return;

    if (selectedDeviceId === -1) {
      TAURI.core.invoke('poll_debug_kbm').then(state => {
        if (currentMode === 'debug') updateDebugState(state);
      }).catch(() => {});
    } else {
      TAURI.core.invoke('poll_gamepad_debug').then(state => {
        if (currentMode === 'debug') updateDebugState(state);
      }).catch(() => {});
    }

    // Refresh device list every ~60 frames (~1s at 60fps)
    debugDeviceRefreshCounter++;
    if (debugDeviceRefreshCounter >= 60) {
      debugDeviceRefreshCounter = 0;
      refreshDebugDeviceList();
    }

    debugPollId = requestAnimationFrame(loop);
  }
  requestAnimationFrame(loop);
}

async function refreshDebugDeviceList() {
  try {
    let devices = await TAURI.core.invoke('list_gamepad_devices');
    let list = document.getElementById('debug-device-list');
    list.innerHTML = '';
    let foundSelected = false;

    // Virtual Keyboard & Mouse device
    let kbmItem = document.createElement('div');
    kbmItem.className = 'debug-device-item';
    kbmItem.dataset.id = '-1';
    kbmItem.innerHTML = `<div class="debug-device-name">Keyboard & Mouse</div>
      <div class="debug-device-id">Virtual  id:-1</div>`;
    if (selectedDeviceId === -1) {
      kbmItem.classList.add('selected');
      foundSelected = true;
    }
    kbmItem.addEventListener('click', () => selectDebugDevice(-1));
    list.appendChild(kbmItem);

    for (let dev of devices) {
      let item = document.createElement('div');
      item.className = 'debug-device-item';
      item.dataset.id = dev.instance_id;
      let tag = dev.is_gamepad ? 'Gamepad' : 'Joystick';
      item.innerHTML = `<div class="debug-device-name">${escHtml(dev.name)}</div>
        <div class="debug-device-id">VID:${pad4(dev.vendor)} PID:${pad4(dev.product)}  ${tag}  id:${dev.instance_id}</div>`;
      if (dev.instance_id === selectedDeviceId) {
        item.classList.add('selected');
        foundSelected = true;
      }
      item.addEventListener('click', () => selectDebugDevice(dev.instance_id));
      list.appendChild(item);
    }
    if (!foundSelected && selectedDeviceId !== null) {
      // Previously selected device is gone
      selectedDeviceId = null;
      document.getElementById('debug-state').textContent = 'Device disconnected — select a device';
    }
  } catch (_) {}
}

async function selectDebugDevice(instanceId) {
  selectedDeviceId = instanceId;
  debugMaxBtns = 0;
  document.querySelectorAll('.debug-device-item').forEach(el => {
    el.classList.toggle('selected', parseInt(el.dataset.id) === instanceId);
  });
  if (instanceId === -1) {
    document.getElementById('debug-state').textContent = 'Listening for keyboard & mouse…';
    return;
  }
  try {
    await TAURI.core.invoke('open_debug_gamepad', { instanceId });
    document.getElementById('debug-state').textContent = 'Opening device…';
  } catch (e) {
    document.getElementById('debug-state').textContent = 'Failed to open device: ' + e;
  }
}

let debugMaxBtns = 0;

function updateDebugState(state) {
  let el = document.getElementById('debug-state');
  if (!state.connected) {
    debugMaxBtns = 0;
    el.textContent = 'Device disconnected';
    return;
  }

  let lines = [];
  lines.push(state.name);

  let isKbm = (state.keyboard && state.keyboard.length > 0) ||
              (state.mouse_buttons && state.mouse_buttons.length > 0) ||
              state.name === 'Keyboard & Mouse';

  if (!isKbm) {
    // Track highest button count ever seen so buttons beyond SDL's
    // joystick count (e.g. gamepad-mapped L5/R5 at 18/19) persist.
    if (state.num_buttons > debugMaxBtns) debugMaxBtns = state.num_buttons;
    let numBtns = debugMaxBtns;

    lines.push(`Vendor: ${pad4(state.vendor)} Product: ${pad4(state.product)}`);
    lines.push(`Type: ${state.is_gamepad ? 'Gamepad' : 'Joystick'}`);
    lines.push(`Buttons: ${numBtns}  Axes: ${state.num_axes}`);
    lines.push('');

    for (let i = 0; i < numBtns && i < state.buttons.length; i++) {
      lines.push(`Button ${i}: ${state.buttons[i] ? 1 : 0}`);
    }
    lines.push('');

    for (let i = 0; i < state.num_axes && i < state.axes.length; i++) {
      lines.push(`Axis ${i}: ${state.axes[i].toFixed(4)}`);
    }

    // Touchpads (SDL_GamepadTouchpad)
    if (state.touchpads && state.touchpads.length > 0) {
      lines.push('');
      state.touchpads.forEach((tp, i) => {
        let name = i === 0 ? 'Left Touchpad' : i === 1 ? 'Right Touchpad' : `Touchpad ${i}`;
        lines.push(`${name} [${i}]: ${tp.down ? 'TOUCH' : 'idle'}  x:${tp.x.toFixed(3)}  y:${tp.y.toFixed(3)}  press:${tp.pressure.toFixed(5)}`);
      });
    }

    // Hats (D-pad as hat switch)
    if (state.hats && state.hats.length > 0) {
      lines.push('');
      let hatNames = ['CENTERED', 'UP', 'RIGHT', 'RIGHTUP', 'DOWN', 'DOWNRIGHT', '?', '?', 'LEFT', 'LEFTUP', '?', '?', 'LEFTDOWN', '?', '?', '?'];
      state.hats.forEach((h, i) => {
        let name = hatNames[h] || `0x${h.toString(16)}`;
        lines.push(`Hat ${i}: ${name} (0x${h.toString(16).padStart(2, '0')})`);
      });
    }

    // Cap sense
    let cs = state.cap_sense;
    if (cs) {
      let cap_names = ['LeftGrip', 'RightGrip', 'LeftStick', 'RightStick'];
      lines.push('');
      lines.push('Cap sense (SDL_GamepadCapSenseType):');
      for (let i = 0; i < cap_names.length && i < cs.length; i++) {
        lines.push(`  [${i}] ${cap_names[i]}: ${cs[i] ? 'ON' : 'OFF'}`);
      }
    }
  }

  // Keyboard state
  if (state.keyboard && state.keyboard.length > 0) {
    lines.push('');
    lines.push('Keyboard keys:');
    for (let k of state.keyboard) {
      lines.push(`  ${k}`);
    }
  }

  // Mouse buttons
  if (state.mouse_buttons && state.mouse_buttons.length > 0) {
    lines.push('');
    lines.push('Mouse buttons:');
    for (let b of state.mouse_buttons) {
      let name = b === 0 ? 'Left' : b === 1 ? 'Middle' : b === 2 ? 'Right' : `Button ${b}`;
      lines.push(`  ${name} (${b})`);
    }
  }

  // Mouse position
  if (state.mouse && (Math.abs(state.mouse[0]) > 0.001 || Math.abs(state.mouse[1]) > 0.001)) {
    lines.push('');
    lines.push(`Mouse: x=${state.mouse[0].toFixed(4)}  y=${state.mouse[1].toFixed(4)}`);
  }

  if (state.scroll && (state.scroll[0] > 0 || state.scroll[1] > 0)) {
    lines.push('');
    lines.push(`Scroll: up=${state.scroll[0]}  down=${state.scroll[1]}`);
  }

  if (isKbm && !state.keyboard?.length && !state.mouse_buttons?.length) {
    lines.push('');
    lines.push('No keys or buttons pressed');
  }

  el.textContent = lines.join('\n');
}

function escHtml(s) {
  let d = document.createElement('div');
  d.textContent = s;
  return d.innerHTML;
}

function pad4(n) {
  return n.toString(16).toUpperCase().padStart(4, '0');
}

// ─── Overlay ───

const DEFAULT_WINDOW_SIZE = { width: 890, height: 681 };

let gearHideTimer = null;

async function showOverlay(def, mode, path, savedColor) {
  currentMode = 'overlay';
  document.getElementById('menu-screen').style.display = 'none';
  document.getElementById('debug-screen').style.display = 'none';
  document.getElementById('overlay-screen').style.display = 'block';
  document.body.style.background = 'transparent';

  document.getElementById('overlay-screen').dataset.controllerPath = path;

  if (savedColor) {
    setPaintColor(savedColor);
    let rgb = savedColor.length >= 9 ? savedColor.slice(0, 7) : savedColor;
    let a = savedColor.length >= 9 ? parseInt(savedColor.slice(7, 9), 16) : 255;
    document.getElementById('paint-color').value = rgb;
    document.getElementById('paint-alpha').value = a;
    document.getElementById('paint-alpha-label').textContent = Math.round(a / 255 * 100) + '%';
  } else {
    // Ensure default alpha is full when no saved color
    let defaultHex = currentColorHex();
    setPaintColor(defaultHex);
  }

  // Load base image and set canvas size
  let dimensions = await initOverlay(def, mode, path);

  // Set window decorations/always-on-top
  if (TAURI) {
    try {
      let win = TAURI.window.getCurrentWindow();
      await win.setDecorations(false);
    } catch (e) { /* ignore */ }
  }

  // Show gear button briefly, then auto-hide on idle
  let gear = document.getElementById('gear-btn');
  gear.style.display = 'block';
  gear.style.opacity = '1';
  scheduleGearHide(2000);

  // Show gear on mouse move, re-hide on idle
  let ov = document.getElementById('overlay-screen');
  ov.addEventListener('mousemove', onOverlayMove);

  // Detect whether this controller uses keyboard/mouse input instead of SDL gamepad
  let useKbmPoll = def.buttons.some(b => b.input.type === 'keyboard' || b.input.type === 'mouse' || b.input.type === 'mouse_move' || b.input.type === 'scroll_wheel');

  // Start polling loop
  function pollLoop() {
    if (currentMode !== 'overlay') return;
    let promise = useKbmPoll
      ? TAURI.core.invoke('poll_debug_kbm')
      : TAURI.core.invoke('poll_gamepad');
    promise.then(state => {
      gamepadState = state;
      renderFrame(state);
    }).catch(e => {
      console.error('poll error:', e);
    });
    requestAnimationFrame(pollLoop);
  }
  requestAnimationFrame(pollLoop);
}

function scheduleGearHide(delay) {
  if (gearHideTimer) clearTimeout(gearHideTimer);
  gearHideTimer = setTimeout(() => {
    let gear = document.getElementById('gear-btn');
    if (gear) gear.style.opacity = '0';
  }, delay);
}

function onOverlayMove() {
  let gear = document.getElementById('gear-btn');
  if (gear) gear.style.opacity = '1';
  scheduleGearHide(2000);
}

// ─── Settings ───

document.getElementById('gear-btn').addEventListener('click', () => {
  let panel = document.getElementById('settings-panel');
  panel.style.display = panel.style.display === 'none' ? 'block' : 'none';
});

document.getElementById('close-settings').addEventListener('click', () => {
  document.getElementById('settings-panel').style.display = 'none';
});

function currentColorHex() {
  let rgb = document.getElementById('paint-color').value;
  let a = parseInt(document.getElementById('paint-alpha').value);
  return rgb + a.toString(16).toUpperCase().padStart(2, '0');
}

document.getElementById('paint-color').addEventListener('input', persistColor);
document.getElementById('paint-alpha').addEventListener('input', () => {
  let a = parseInt(document.getElementById('paint-alpha').value);
  document.getElementById('paint-alpha-label').textContent = Math.round(a / 255 * 100) + '%';
  persistColor();
});

function persistColor() {
  let hex = currentColorHex();
  setPaintColor(hex);
  let path = document.getElementById('overlay-screen').dataset.controllerPath;
  if (path) {
    TAURI.core.invoke('persist_color', { path, color: hex }).catch(() => {});
  }
}

document.getElementById('back-to-menu').addEventListener('click', async () => {
  currentMode = 'menu';
  document.getElementById('settings-panel').style.display = 'none';
  document.getElementById('gear-btn').style.display = 'none';
  document.getElementById('gear-btn').style.opacity = '1';
  document.getElementById('overlay-screen').removeEventListener('mousemove', onOverlayMove);
  if (gearHideTimer) { clearTimeout(gearHideTimer); gearHideTimer = null; }
  document.getElementById('overlay-screen').style.display = 'none';
  document.getElementById('debug-screen').style.display = 'none';
  document.getElementById('menu-screen').style.display = 'flex';
  document.body.style.background = '#141414';

  // Reset window to menu size
  if (TAURI) {
    try {
      console.log('resizing to default', DEFAULT_WINDOW_SIZE);
      await TAURI.core.invoke('resize_window', DEFAULT_WINDOW_SIZE).catch(e => console.error('resize error:', e));
      await new Promise(r => requestAnimationFrame(r));
      await TAURI.core.invoke('resize_window', DEFAULT_WINDOW_SIZE).catch(e => console.error('resize error:', e));
    } catch (e) { /* ignore */ }
  }

  await showMenu();
});
