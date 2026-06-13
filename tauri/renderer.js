let canvas = document.getElementById('overlay-canvas');
let ctx = canvas.getContext('2d');

let controllerDef = null;
let controllerPath = null;
let controllerMode = null;
let images = {};       // id -> HTMLImageElement
let baseImage = null;
let gamepadState = null;
let paintColor = '#00c8ff';
let animFrameId = null;

// Tap paint state: button id → frames remaining
let tapTimers = {};
let prevTapStates = {};

// Scroll wheel flash timers: button id → frames remaining
let scrollUpTimers = {};
let scrollDownTimers = {};
let prevScrollState = {};

// Cap sense analog smoothing
let smoothedCapSense = [0, 0, 0, 0];
const CAPSENSE_LERP = 0.12;

/** Resolve the effective image path for a button, applying any mode override. */
function buttonImage(btn) {
  if (controllerMode && controllerDef.mode_overrides) {
    let modeOv = controllerDef.mode_overrides[controllerMode];
    if (modeOv && modeOv.buttons && modeOv.buttons[btn.id] && modeOv.buttons[btn.id].image) {
      return modeOv.buttons[btn.id].image;
    }
  }
  return btn.image;
}

export async function initOverlay(def, mode, path) {
  controllerDef = def;
  controllerMode = mode;
  controllerPath = path;
  images = {};
  baseImage = null;
  tapTimers = {};
  prevTapStates = {};
  scrollUpTimers = {};
  scrollDownTimers = {};
  prevScrollState = {};
  smoothedCapSense = [0, 0, 0, 0];

  let basePath = def.controller.base;
  let baseImg = await loadImage('get_base_image', basePath);
  if (baseImg) {
    baseImage = baseImg;
    canvas.width = baseImg.naturalWidth;
    canvas.height = baseImg.naturalHeight;
  }

  for (let btn of def.buttons) {
    let imgPath = buttonImage(btn);
    if (imgPath && !images[imgPath]) {
      images[imgPath] = null;
      loadImage('get_button_image', imgPath).then(img => {
        images[imgPath] = img;
      });
    }
    if (btn.image_up && !images[btn.image_up]) {
      images[btn.image_up] = null;
      loadImage('get_button_image', btn.image_up).then(img => {
        images[btn.image_up] = img;
      });
    }
    if (btn.image_down && !images[btn.image_down]) {
      images[btn.image_down] = null;
      loadImage('get_button_image', btn.image_down).then(img => {
        images[btn.image_down] = img;
      });
    }
  }

  return baseImg ? { width: baseImg.naturalWidth, height: baseImg.naturalHeight } : null;
}

async function loadImage(command, imagePath) {
  try {
    let dataUrl = await window.__TAURI__.core.invoke(command, {
      path: controllerPath,
      image: imagePath,
    });
    let img = new Image();
    img.src = dataUrl;
    await img.decode();
    return img;
  } catch (e) {
    console.error('Failed to load image:', imagePath, e);
    return null;
  }
}

export function renderFrame(state) {
  gamepadState = state;
  if (animFrameId) return;
  animFrameId = requestAnimationFrame(drawFrame);
}

function drawFrame() {
  animFrameId = null;
  if (!controllerDef || !gamepadState) return;

  // Decay tap timers
  for (let id in tapTimers) {
    tapTimers[id] = Math.max(0, tapTimers[id] - 1);
  }

  // Decay scroll wheel flash timers
  for (let id in scrollUpTimers) {
    scrollUpTimers[id] = Math.max(0, scrollUpTimers[id] - 1);
  }
  for (let id in scrollDownTimers) {
    scrollDownTimers[id] = Math.max(0, scrollDownTimers[id] - 1);
  }

  // Process scroll events from gamepad state
  if (gamepadState.scroll) {
    for (let btn of controllerDef.buttons) {
      if (!btn.image_up && !btn.image_down) continue;
      let scroll = gamepadState.scroll;
      if (scroll[0] > 0 && !prevScrollState[btn.id + '_up']) {
        scrollUpTimers[btn.id] = 6;
      }
      if (scroll[1] > 0 && !prevScrollState[btn.id + '_down']) {
        scrollDownTimers[btn.id] = 6;
      }
      prevScrollState[btn.id + '_up'] = scroll[0] > 0;
      prevScrollState[btn.id + '_down'] = scroll[1] > 0;
    }
  } else {
    prevScrollState = {};
  }

  // Smooth cap sense analog values toward boolean target
  for (let i = 0; i < 4; i++) {
    let target = (gamepadState.cap_sense && gamepadState.cap_sense[i]) ? 1.0 : 0.0;
    smoothedCapSense[i] += (target - smoothedCapSense[i]) * CAPSENSE_LERP;
  }

  let w = canvas.width;
  let h = canvas.height;
  ctx.clearRect(0, 0, w, h);

  if (baseImage) {
    ctx.drawImage(baseImage, 0, 0, w, h);
  }

  let btns = controllerDef.buttons;

  // Build reference map: targetId → [modifier buttons]
  let refMap = {};
  for (let btn of btns) {
    if (btn.reference) {
      if (!refMap[btn.reference]) refMap[btn.reference] = [];
      refMap[btn.reference].push(btn);
    }
  }

  // Find indices covered by active multi-button combos
  let suppressed = new Set();
  for (let btn of btns) {
    if (btn.input.type === 'multi' && isPressed(btn, gamepadState)) {
      for (let i of btn.input.indices) suppressed.add(i);
    }
  }

  // First pass: all non-stick buttons (skip reference entries —
  // their paint behaviors are merged into the target via modifiers)
  for (let btn of btns) {
    if (btn.reference || btn.input.type === 'stick') continue;
    drawButton(btn, 0, 0, refMap[btn.id] || [], suppressed);
  }

  // Second pass: stick caps with axis offset
  for (let btn of btns) {
    if (btn.reference || btn.input.type !== 'stick') continue;
    let ax = gamepadState.axes[btn.input.axis_x] || 0;
    let ay = gamepadState.axes[btn.input.axis_y] || 0;
    let ox = ax * btn.travel;
    let oy = ay * btn.travel;
    drawButton(btn, ox, oy, refMap[btn.id] || []);
  }
}

function getAnalogValue(btn, state) {
  if (btn.capsense != null) {
    return smoothedCapSense[btn.capsense] || 0;
  }
  switch (btn.input.type) {
    case 'trigger':
      return Math.max(0, Math.min(1, state.axes[btn.input.axis] || 0));
    case 'button':
      return state.buttons[btn.input.index] ? 1 : 0;
    case 'stick':
      return state.buttons[btn.input.press_button] ? 1 : 0;
    case 'touchpad': {
      let finger = state.touchpads && state.touchpads[btn.input.index];
      return finger && finger.down ? finger.pressure : 0;
    }
    default:
      return 0;
  }
}

function getPaintAlpha(btn, state, globalAlpha) {
  let maxAlpha = (btn.transparency != null ? btn.transparency : 1.0) * globalAlpha;
  let paint = btn.paint || 'standard';

  if (paint === 'none' || paint === 'trackpad' || paint === 'mouse') return 0;

  if (paint === 'inverse') {
    let active;
    if (btn.capsense != null) {
      active = state.cap_sense && state.cap_sense[btn.capsense] || false;
    } else {
      active = isPressed(btn, state);
    }
    return active ? 0 : maxAlpha;
  }

  if (paint === 'tap') {
    let curr;
    if (btn.capsense != null) {
      curr = state.cap_sense && state.cap_sense[btn.capsense] || false;
    } else {
      curr = isPressed(btn, state);
    }
    let prev = prevTapStates[btn.id] || false;
    prevTapStates[btn.id] = curr;
    if (curr && !prev) {
      tapTimers[btn.id] = 4;
    }
    return (tapTimers[btn.id] || 0) > 0 ? maxAlpha : 0;
  }

  if (paint === 'progressive') {
    let analogVal = getAnalogValue(btn, state);
    if (analogVal <= 0) return 0;
    return analogVal * maxAlpha;
  }

  let pressed = isPressed(btn, state);
  if (!pressed) return 0;

  // standard
  return maxAlpha;
}

function drawButton(btn, offsetX = 0, offsetY = 0, modifiers = [], suppressed = new Set()) {
  let img = images[btn.image];
  let isMouse = btn.paint === 'mouse';
  let isScroll = btn.image_up || btn.image_down;
  if (!img && !isMouse && !isScroll) return;

  let x = offsetX;
  let y = offsetY;
  let w = canvas.width;
  let h = canvas.height;
  let globalAlpha = hexAlpha(paintColor);
  let alpha = getPaintAlpha(btn, gamepadState, globalAlpha);

  for (let mod of modifiers) {
    let ma = getPaintAlpha(mod, gamepadState, globalAlpha);
    if (ma > alpha) alpha = ma;
  }
  let doPaint = alpha > 0.001;
  // Suppress paint (but not the base image) when a multi combo covers this index
  if (doPaint && btn.input.type === 'button' && suppressed.has(btn.input.index)) {
    doPaint = false;
  }

  if (img) {
    if (doPaint) {
      let temp = document.createElement('canvas');
      temp.width = w;
      temp.height = h;
      let tc = temp.getContext('2d');
      tc.drawImage(img, 0, 0, w, h);

      let d = tc.getImageData(0, 0, w, h);
      let pr = parseInt(paintColor.slice(1, 3), 16);
      let pg = parseInt(paintColor.slice(3, 5), 16);
      let pb = parseInt(paintColor.slice(5, 7), 16);
      let inv = 1 - alpha;
      for (let i = 0; i < d.data.length; i += 4) {
        if (d.data[i + 3] > 0) {
          d.data[i    ] = d.data[i    ] * inv + pr * alpha;
          d.data[i + 1] = d.data[i + 1] * inv + pg * alpha;
          d.data[i + 2] = d.data[i + 2] * inv + pb * alpha;
        }
      }
      tc.putImageData(d, 0, 0);
      ctx.drawImage(temp, x, y);
    } else {
      ctx.drawImage(img, x, y, w, h);
    }
  }

  // Trackpad finger dot — drawn on top at the tracked finger position
  // Check both the button itself and its modifiers
  let trackpadButtons = [btn, ...modifiers];
  for (let tb of trackpadButtons) {
    if (tb.paint !== 'trackpad') continue;
    let finger = gamepadState.touchpads && gamepadState.touchpads[tb.input.index];
    let sensitivity = tb.pad_sensitivity || 0;
    if (finger && finger.down && finger.pressure >= sensitivity) {
      let pos = constrainToPadShape(finger, tb);
      ctx.beginPath();
      ctx.arc(pos.x, pos.y, 12, 0, Math.PI * 2);
      ctx.fillStyle = paintRgba();
      ctx.fill();
    }
  }

  // Mouse dot — virtual stick driven by mouse deltas
  for (let mb of trackpadButtons) {
    if (mb.paint !== 'mouse') continue;
    let mousePos = gamepadState.mouse;
    if (!mousePos) continue;
    let dx = mousePos[0], dy = mousePos[1];
    if (Math.abs(dx) < 0.01 && Math.abs(dy) < 0.01) continue;
    let hw = (mb.pad_size_x || 200) / 2;
    let hh = (mb.pad_size_y || 200) / 2;
    let cx = mb.pad_center_x != null ? mb.pad_center_x : canvas.width / 2;
    let cy = mb.pad_center_y != null ? mb.pad_center_y : canvas.height / 2;
    let px = cx + dx * hw;
    let py = cy + dy * hh;
    ctx.beginPath();
    ctx.arc(px, py, 12, 0, Math.PI * 2);
    ctx.fillStyle = paintRgba();
    ctx.fill();
  }

  // Scroll wheel — always draw both halves; paint overlay on events
  if (btn.image_up || btn.image_down) {
    let pr = parseInt(paintColor.slice(1, 3), 16);
    let pg = parseInt(paintColor.slice(3, 5), 16);
    let pb = parseInt(paintColor.slice(5, 7), 16);
    let ga = hexAlpha(paintColor);
    let flashAlpha = (btn.transparency != null ? btn.transparency : 1.0) * ga;

    let upTimer = scrollUpTimers[btn.id] || 0;
    let downTimer = scrollDownTimers[btn.id] || 0;
    let isClicked = gamepadState && gamepadState.mouse_buttons && gamepadState.mouse_buttons.includes(2);

    let paintHalf = function(overlayImg, doFlash) {
      if (!overlayImg || !images[overlayImg]) return;
      if (doFlash && flashAlpha > 0.001) {
        let temp = document.createElement('canvas');
        temp.width = w;
        temp.height = h;
        let tc = temp.getContext('2d');
        tc.drawImage(images[overlayImg], 0, 0, w, h);
        let d = tc.getImageData(0, 0, w, h);
        let inv = 1 - flashAlpha;
        for (let i = 0; i < d.data.length; i += 4) {
          if (d.data[i + 3] > 0) {
            d.data[i    ] = d.data[i    ] * inv + pr * flashAlpha;
            d.data[i + 1] = d.data[i + 1] * inv + pg * flashAlpha;
            d.data[i + 2] = d.data[i + 2] * inv + pb * flashAlpha;
          }
        }
        tc.putImageData(d, 0, 0);
        ctx.drawImage(temp, 0, 0);
      } else {
        ctx.drawImage(images[overlayImg], 0, 0, w, h);
      }
    };

    paintHalf(btn.image_up, upTimer > 0 || isClicked);
    paintHalf(btn.image_down, downTimer > 0 || isClicked);
  }
}

function clampToShape(lx, ly, halfW, halfH, shape) {
  switch (shape) {
    case 'square':
      return {
        x: Math.max(-halfW, Math.min(halfW, lx)),
        y: Math.max(-halfH, Math.min(halfH, ly))
      };
    case 'circle': {
      if (lx === 0 && ly === 0) return {x: lx, y: ly};
      let dx = lx / halfW;
      let dy = ly / halfH;
      let dist = Math.sqrt(dx * dx + dy * dy);
      if (dist <= 1) return {x: lx, y: ly};
      return {x: lx / dist, y: ly / dist};
    }
    case 'squircle': {
      if (lx === 0 && ly === 0) return {x: lx, y: ly};
      let dx = lx / halfW;
      let dy = ly / halfH;
      let val = Math.pow(Math.abs(dx), 4) + Math.pow(Math.abs(dy), 4);
      if (val <= 1) return {x: lx, y: ly};
      let scale = 1 / Math.pow(val, 0.25);
      return {x: lx * scale, y: ly * scale};
    }
    default:
      return {x: lx, y: ly};
  }
}

function constrainToPadShape(finger, btn) {
  if (btn.pad_size_x == null || btn.pad_size_y == null) {
    let w = canvas.width;
    let h = canvas.height;
    let fx = finger.x * w;
    let fy = finger.y * h;
    return {x: fx, y: fy};
  }
  let hw = btn.pad_size_x / 2;
  let hh = btn.pad_size_y / 2;
  let cx = btn.pad_center_x != null ? btn.pad_center_x : canvas.width / 2;
  let cy = btn.pad_center_y != null ? btn.pad_center_y : canvas.height / 2;

  // Map finger (0-1) to the touchpad bounding box (centered on pad_center)
  let lx = (finger.x - 0.5) * btn.pad_size_x;
  let ly = (finger.y - 0.5) * btn.pad_size_y;

  // Inverse rotate to align with shape axes
  let angle = (btn.pad_angle || 0) * Math.PI / 180;
  let cos = Math.cos(-angle);
  let sin = Math.sin(-angle);
  let rx = lx * cos - ly * sin;
  let ry = lx * sin + ly * cos;

  // Clamp to shape
  let clamped = clampToShape(rx, ry, hw, hh, btn.pad_shape || 'circle');

  // Forward rotate
  cos = Math.cos(angle);
  sin = Math.sin(angle);
  let crx = clamped.x * cos - clamped.y * sin;
  let cry = clamped.x * sin + clamped.y * cos;

  return {x: cx + crx, y: cy + cry};
}

function isPressed(btn, state) {
  switch (btn.input.type) {
    case 'button':
      return state.buttons[btn.input.index] || false;
    case 'trigger': {
      let val = state.axes[btn.input.axis] || 0;
      let thresh = btn.input.threshold || 0.1;
      return val > thresh;
    }
    case 'stick':
      if (btn.input.press_button !== undefined && btn.input.press_button !== null) {
        return state.buttons[btn.input.press_button] || false;
      }
      return false;
    case 'touchpad': {
      if (btn.input.press_button != null) {
        return state.buttons[btn.input.press_button] || false;
      }
      let finger = state.touchpads && state.touchpads[btn.input.index];
      return finger ? finger.down : false;
    }
    case 'keyboard':
      return state.keyboard && state.keyboard.includes(btn.input.key);
    case 'mouse':
      return state.mouse_buttons && state.mouse_buttons.includes(btn.input.button);
    case 'mouse_move':
      return false;
    case 'scroll_wheel':
      return state.mouse_buttons && state.mouse_buttons.includes(2);
    case 'hat':
      return state.hats && state.hats[btn.input.index] !== undefined &&
             (state.hats[btn.input.index] & btn.input.direction) !== 0;
    case 'multi':
      return btn.input.indices && btn.input.indices.every(i => state.buttons[i]);
    default:
      return false;
  }
}

function paintRgba() {
  let r = parseInt(paintColor.slice(1, 3), 16);
  let g = parseInt(paintColor.slice(3, 5), 16);
  let b = parseInt(paintColor.slice(5, 7), 16);
  let a = paintColor.length >= 9 ? parseInt(paintColor.slice(7, 9), 16) / 255 : 1;
  return `rgba(${r},${g},${b},${a.toFixed(3)})`;
}

export function setPaintColor(color) {
  paintColor = color;
}

function hexAlpha(color) {
  return color.length >= 9 ? parseInt(color.slice(7, 9), 16) / 255 : 1;
}
