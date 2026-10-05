-- Omarchy-style Hyprland for NixOS, adapted from Omarchy 3.8.4
-- (https://github.com/basecamp/omarchy, MIT; see ../../LICENSE) for the Lua
-- configuration of Hyprland 0.56. Configuration calls only: dynamic behavior
-- lives in the `omarchy` helper, and session components are systemd user units
-- started with this session. To customize, copy this file (including the
-- keyboard section NixOS appends) to ~/.config/omarchy/hyprland.lua.

hl.monitor({ output = "", mode = "preferred", position = "auto", scale = "auto" })

-------------------------------
---- ENVIRONMENT VARIABLES ----
-------------------------------

hl.env("XCURSOR_SIZE", "24")
hl.env("HYPRCURSOR_SIZE", "24")
hl.env("GDK_BACKEND", "wayland,x11,*")
hl.env("QT_QPA_PLATFORM", "wayland;xcb")
hl.env("MOZ_ENABLE_WAYLAND", "1")
hl.env("ELECTRON_OZONE_PLATFORM_HINT", "wayland")
hl.env("OZONE_PLATFORM", "wayland")
hl.env("XDG_SESSION_TYPE", "wayland")
hl.env("XDG_CURRENT_DESKTOP", "Hyprland")
hl.env("XDG_SESSION_DESKTOP", "Hyprland")
hl.env("TERMINAL", "alacritty")

-----------------------
---- LOOK AND FEEL ----
-----------------------

-- Tokyo Night: the accent is the active border, as in Omarchy's theme.
hl.config({
  general = {
    gaps_in = 5,
    gaps_out = 10,
    border_size = 2,
    col = {
      active_border = "rgb(7aa2f7)",
      inactive_border = "rgba(595959aa)",
    },
    resize_on_border = false,
    allow_tearing = false,
    layout = "dwindle",
  },

  decoration = {
    rounding = 0,
    shadow = {
      enabled = true,
      range = 2,
      render_power = 3,
      color = "rgba(1a1a1aee)",
    },
    blur = {
      enabled = true,
      size = 2,
      passes = 2,
      special = true,
      brightness = 0.60,
      contrast = 0.75,
    },
  },

  group = {
    col = {
      border_active = "rgb(7aa2f7)",
      border_inactive = "rgba(595959aa)",
      border_locked_active = "rgb(7aa2f7)",
      border_locked_inactive = "rgba(595959aa)",
    },
    groupbar = {
      font_size = 12,
      font_family = "JetBrainsMono Nerd Font",
      font_weight_active = "ultraheavy",
      font_weight_inactive = "normal",
      indicator_height = 0,
      indicator_gap = 5,
      height = 22,
      gaps_in = 5,
      gaps_out = 0,
      text_color = "rgb(ffffff)",
      text_color_inactive = "rgba(ffffff90)",
      col = {
        active = "rgba(00000040)",
        inactive = "rgba(00000020)",
      },
      gradients = true,
      gradient_rounding = 0,
      gradient_round_only_edges = false,
    },
  },

  animations = {
    enabled = true,
  },

  dwindle = {
    preserve_split = true,
    force_split = 2,
  },

  scrolling = {
    column_width = 0.49,
  },

  master = {
    new_status = "master",
  },

  misc = {
    disable_hyprland_logo = true,
    disable_splash_rendering = true,
    disable_scale_notification = true,
    focus_on_activate = true,
    anr_missed_pings = 3,
    on_focus_under_fullscreen = 1,
    key_press_enables_dpms = true,
    mouse_move_enables_dpms = true,
    background_color = "rgb(1a1b26)",
  },

  cursor = {
    hide_on_key_press = true,
    warp_on_change_workspace = 1,
  },

  binds = {
    hide_special_on_workspace_change = true,
  },

  xwayland = {
    force_zero_scaling = true,
  },

  ecosystem = {
    no_update_news = true,
    no_donation_nag = true,
  },
})

hl.curve("easeOutQuint", { type = "bezier", points = { { 0.23, 1 }, { 0.32, 1 } } })
hl.curve("easeInOutCubic", { type = "bezier", points = { { 0.65, 0.05 }, { 0.36, 1 } } })
hl.curve("linear", { type = "bezier", points = { { 0, 0 }, { 1, 1 } } })
hl.curve("almostLinear", { type = "bezier", points = { { 0.5, 0.5 }, { 0.75, 1.0 } } })
hl.curve("quick", { type = "bezier", points = { { 0.15, 0 }, { 0.1, 1 } } })

hl.animation({ leaf = "global", enabled = true, speed = 10, bezier = "default" })
hl.animation({ leaf = "border", enabled = true, speed = 5.39, bezier = "easeOutQuint" })
hl.animation({ leaf = "windows", enabled = true, speed = 3.79, bezier = "easeOutQuint" })
hl.animation({ leaf = "windowsIn", enabled = true, speed = 4.1, bezier = "easeOutQuint", style = "popin 87%" })
hl.animation({ leaf = "windowsOut", enabled = true, speed = 1.49, bezier = "linear", style = "popin 87%" })
hl.animation({ leaf = "fadeIn", enabled = true, speed = 1.73, bezier = "almostLinear" })
hl.animation({ leaf = "fadeOut", enabled = true, speed = 1.46, bezier = "almostLinear" })
hl.animation({ leaf = "fade", enabled = true, speed = 3.03, bezier = "quick" })
hl.animation({ leaf = "layers", enabled = true, speed = 3.81, bezier = "easeOutQuint" })
hl.animation({ leaf = "layersIn", enabled = true, speed = 4, bezier = "easeOutQuint", style = "fade" })
hl.animation({ leaf = "layersOut", enabled = true, speed = 1.5, bezier = "linear", style = "fade" })
hl.animation({ leaf = "fadeLayersIn", enabled = true, speed = 1.79, bezier = "almostLinear" })
hl.animation({ leaf = "fadeLayersOut", enabled = true, speed = 1.39, bezier = "almostLinear" })
hl.animation({ leaf = "workspaces", enabled = false })
hl.animation({ leaf = "specialWorkspace", enabled = true, speed = 3, bezier = "easeOutQuint", style = "slidevert" })

---------------
---- INPUT ----
---------------

-- The keyboard layout of the installed system is appended below by NixOS.
hl.config({
  input = {
    follow_mouse = 1,
    sensitivity = 0,
    repeat_rate = 40,
    repeat_delay = 250,
    numlock_by_default = true,
    touchpad = {
      natural_scroll = false,
      clickfinger_behavior = true,
      scroll_factor = 0.4,
    },
  },
})

--------------------------------
---- WINDOWS AND WORKSPACES ----
--------------------------------

hl.window_rule({ match = { class = ".*" }, suppress_event = "maximize" })
hl.window_rule({ match = { class = ".*" }, tag = "+default-opacity" })
-- Fix some dragging issues with XWayland.
hl.window_rule({
  match = { class = "^$", title = "^$", xwayland = true, float = true, fullscreen = false, pin = false },
  no_focus = true,
})

-- Floating utility windows, including the helper's terminal applications.
hl.window_rule({
  match = { class = "(org.omarchy\\..*|org.gnome.NautilusPreviewer|org.gnome.Evince|com.gabm.satty|imv|mpv)" },
  tag = "+floating-window",
})
hl.window_rule({
  match = {
    class = "(xdg-desktop-portal-gtk|org.gnome.Nautilus)",
    title = "^(Open.*Files?|Open [F|f]older.*|Save.*Files?|Save.*As|Save|All Files|.*wants to [open|save].*|[C|c]hoose.*)",
  },
  tag = "+floating-window",
})
hl.window_rule({ match = { tag = "floating-window" }, float = true, center = true, size = { 875, 600 } })
hl.window_rule({ match = { class = "org.omarchy.about" }, size = { 920, 520 } })
hl.window_rule({ match = { class = "org.gnome.Calculator" }, float = true })

-- No transparency on media windows.
hl.window_rule({
  match = { class = "^(zoom|vlc|mpv|org.kde.kdenlive|com.obsproject.Studio|com.github.PintaProject.Pinta|imv|org.gnome.NautilusPreviewer)$" },
  tag = "-default-opacity",
  opacity = "1 1",
})

-- Terminals.
hl.window_rule({ match = { class = "(Alacritty|kitty|com.mitchellh.ghostty|foot|org.omarchy\\..*)" }, tag = "+terminal" })
hl.window_rule({ match = { tag = "terminal" }, tag = "-default-opacity", opacity = "0.97 0.9" })
hl.window_rule({ match = { class = "(Alacritty|kitty|foot)" }, scroll_touchpad = 1.5 })

-- Browsers stay opaque apart from a subtle unfocused fade.
hl.window_rule({ match = { class = "((google-)?[cC]hrom(e|ium)|[bB]rave-browser|[mM]icrosoft-edge|Vivaldi-stable)" }, tag = "+chromium-based-browser" })
hl.window_rule({ match = { class = "([fF]irefox|zen|librewolf)" }, tag = "+firefox-based-browser" })
hl.window_rule({ match = { tag = "chromium-based-browser" }, tag = "-default-opacity", tile = true, opacity = "1.0 0.97" })
hl.window_rule({ match = { tag = "firefox-based-browser" }, tag = "-default-opacity", opacity = "1.0 0.97" })

-- Picture-in-picture overlays.
hl.window_rule({ match = { title = "(Picture.?in.?[Pp]icture)" }, tag = "+pip" })
hl.window_rule({
  match = { tag = "pip" },
  tag = "-default-opacity",
  float = true,
  pin = true,
  size = { 600, 338 },
  keep_aspect_ratio = true,
  border_size = 0,
  opacity = "1 1",
  move = { "(monitor_w-window_w-40)", "(monitor_h*0.04)" },
})

-- Prevent idle while open.
hl.window_rule({ match = { tag = "noidle" }, idle_inhibit = "always" })

-- Default opacity for everything that did not opt out above.
hl.window_rule({ match = { tag = "default-opacity" }, opacity = "0.97 0.9" })

hl.layer_rule({ match = { namespace = "walker" }, no_anim = true })

---------------------
---- KEYBINDINGS ----
---------------------

-- Applications
hl.bind("SUPER + RETURN", hl.dsp.exec_cmd("omarchy terminal"), { description = "Terminal" })
hl.bind("SUPER + SHIFT + RETURN", hl.dsp.exec_cmd("omarchy browser"), { description = "Browser" })
hl.bind("SUPER + SHIFT + B", hl.dsp.exec_cmd("omarchy browser"), { description = "Browser" })
hl.bind("SUPER + SHIFT + ALT + B", hl.dsp.exec_cmd("omarchy browser --private"), { description = "Browser (private)" })
hl.bind("SUPER + SHIFT + F", hl.dsp.exec_cmd("nautilus --new-window"), { description = "File manager" })

-- Menus
hl.bind("SUPER + SPACE", hl.dsp.exec_cmd("walker"), { description = "Launch apps" })
hl.bind("SUPER + CTRL + E", hl.dsp.exec_cmd("walker -m symbols"), { description = "Emoji picker" })
hl.bind("SUPER + CTRL + V", hl.dsp.exec_cmd("walker -m clipboard"), { description = "Clipboard manager" })
hl.bind("SUPER + ALT + SPACE", hl.dsp.exec_cmd("omarchy menu"), { description = "Menu" })
hl.bind("SUPER + CTRL + C", hl.dsp.exec_cmd("omarchy menu capture"), { description = "Capture menu" })
hl.bind("SUPER + CTRL + O", hl.dsp.exec_cmd("omarchy menu toggle"), { description = "Toggle menu" })
hl.bind("SUPER + ESCAPE", hl.dsp.exec_cmd("omarchy menu system"), { description = "System menu" })
hl.bind("XF86PowerOff", hl.dsp.exec_cmd("omarchy menu system"), { locked = true, description = "Power menu" })
hl.bind("SUPER + K", hl.dsp.exec_cmd("omarchy keybindings"), { description = "Show key bindings" })
hl.bind("XF86Calculator", hl.dsp.exec_cmd("gnome-calculator"), { description = "Calculator" })

-- Aesthetics and notifications
hl.bind("SUPER + SHIFT + SPACE", hl.dsp.exec_cmd("omarchy toggle bar"), { description = "Toggle top bar" })
hl.bind("SUPER + COMMA", hl.dsp.exec_cmd("makoctl dismiss"), { description = "Dismiss last notification" })
hl.bind("SUPER + SHIFT + COMMA", hl.dsp.exec_cmd("makoctl dismiss --all"), { description = "Dismiss all notifications" })
hl.bind("SUPER + CTRL + COMMA", hl.dsp.exec_cmd("omarchy toggle notifications"), { description = "Toggle silencing notifications" })
hl.bind("SUPER + ALT + COMMA", hl.dsp.exec_cmd("makoctl invoke"), { description = "Invoke last notification" })
hl.bind("SUPER + SHIFT + ALT + COMMA", hl.dsp.exec_cmd("makoctl restore"), { description = "Restore last notification" })
hl.bind("SUPER + CTRL + I", hl.dsp.exec_cmd("omarchy toggle idle"), { description = "Toggle locking on idle" })
hl.bind("SUPER + CTRL + N", hl.dsp.exec_cmd("omarchy toggle nightlight"), { description = "Toggle nightlight" })

-- Captures
hl.bind("PRINT", hl.dsp.exec_cmd("omarchy screenshot"), { description = "Screenshot" })
hl.bind("SUPER + PRINT", hl.dsp.exec_cmd("omarchy color-picker"), { description = "Color picker" })

-- Control panels and system
hl.bind("SUPER + CTRL + A", hl.dsp.exec_cmd("omarchy tui wiremix"), { description = "Audio controls" })
hl.bind("SUPER + CTRL + B", hl.dsp.exec_cmd("omarchy tui bluetui"), { description = "Bluetooth controls" })
hl.bind("SUPER + CTRL + W", hl.dsp.exec_cmd("omarchy tui nmtui"), { description = "Wifi controls" })
hl.bind("SUPER + CTRL + T", hl.dsp.exec_cmd("omarchy tui btop"), { description = "Activity" })
hl.bind("SUPER + CTRL + L", hl.dsp.exec_cmd("omarchy lock"), { description = "Lock system" })

-- Media keys with on-screen display
hl.bind("XF86AudioRaiseVolume", hl.dsp.exec_cmd("omarchy volume raise"), { locked = true, repeating = true, description = "Volume up" })
hl.bind("XF86AudioLowerVolume", hl.dsp.exec_cmd("omarchy volume lower"), { locked = true, repeating = true, description = "Volume down" })
hl.bind("XF86AudioMute", hl.dsp.exec_cmd("omarchy volume mute-toggle"), { locked = true, description = "Mute" })
hl.bind("XF86AudioMicMute", hl.dsp.exec_cmd("omarchy mic-mute"), { locked = true, description = "Mute microphone" })
hl.bind("XF86MonBrightnessUp", hl.dsp.exec_cmd("omarchy brightness +5%"), { locked = true, repeating = true, description = "Brightness up" })
hl.bind("XF86MonBrightnessDown", hl.dsp.exec_cmd("omarchy brightness 5%-"), { locked = true, repeating = true, description = "Brightness down" })
hl.bind("SHIFT + XF86MonBrightnessUp", hl.dsp.exec_cmd("omarchy brightness 100%"), { locked = true, description = "Brightness maximum" })
hl.bind("SHIFT + XF86MonBrightnessDown", hl.dsp.exec_cmd("omarchy brightness 1%"), { locked = true, description = "Brightness minimum" })
hl.bind("ALT + XF86AudioRaiseVolume", hl.dsp.exec_cmd("omarchy volume +1"), { locked = true, repeating = true, description = "Volume up precise" })
hl.bind("ALT + XF86AudioLowerVolume", hl.dsp.exec_cmd("omarchy volume -1"), { locked = true, repeating = true, description = "Volume down precise" })
hl.bind("ALT + XF86MonBrightnessUp", hl.dsp.exec_cmd("omarchy brightness +1%"), { locked = true, repeating = true, description = "Brightness up precise" })
hl.bind("ALT + XF86MonBrightnessDown", hl.dsp.exec_cmd("omarchy brightness 1%-"), { locked = true, repeating = true, description = "Brightness down precise" })
hl.bind("XF86AudioNext", hl.dsp.exec_cmd("omarchy media next"), { locked = true, description = "Next track" })
hl.bind("XF86AudioPause", hl.dsp.exec_cmd("omarchy media play-pause"), { locked = true, description = "Pause" })
hl.bind("XF86AudioPlay", hl.dsp.exec_cmd("omarchy media play-pause"), { locked = true, description = "Play" })
hl.bind("XF86AudioPrev", hl.dsp.exec_cmd("omarchy media previous"), { locked = true, description = "Previous track" })

-- Universal clipboard
hl.bind("SUPER + C", hl.dsp.send_shortcut({ mods = "CTRL", key = "Insert" }), { description = "Universal copy" })
hl.bind("SUPER + V", hl.dsp.send_shortcut({ mods = "SHIFT", key = "Insert" }), { description = "Universal paste" })
hl.bind("SUPER + X", hl.dsp.send_shortcut({ mods = "CTRL", key = "X" }), { description = "Universal cut" })

-- Windows
hl.bind("SUPER + W", hl.dsp.window.close(), { description = "Close window" })
hl.bind("CTRL + ALT + DELETE", hl.dsp.exec_cmd("omarchy close-all"), { description = "Close all windows" })
hl.bind("SUPER + J", hl.dsp.layout("togglesplit"), { description = "Toggle window split" })
hl.bind("SUPER + P", hl.dsp.window.pseudo(), { description = "Pseudo window" })
hl.bind("SUPER + T", hl.dsp.window.float({ action = "toggle" }), { description = "Toggle window floating/tiling" })
hl.bind("SUPER + F", hl.dsp.window.fullscreen({ mode = "fullscreen" }), { description = "Full screen" })
hl.bind("SUPER + ALT + F", hl.dsp.window.fullscreen({ mode = "maximized" }), { description = "Full width" })

hl.bind("SUPER + LEFT", hl.dsp.focus({ direction = "l" }), { description = "Focus on left window" })
hl.bind("SUPER + RIGHT", hl.dsp.focus({ direction = "r" }), { description = "Focus on right window" })
hl.bind("SUPER + UP", hl.dsp.focus({ direction = "u" }), { description = "Focus on above window" })
hl.bind("SUPER + DOWN", hl.dsp.focus({ direction = "d" }), { description = "Focus on below window" })

hl.bind("SUPER + code:10", hl.dsp.focus({ workspace = "1" }), { description = "Switch to workspace 1" })
hl.bind("SUPER + code:11", hl.dsp.focus({ workspace = "2" }), { description = "Switch to workspace 2" })
hl.bind("SUPER + code:12", hl.dsp.focus({ workspace = "3" }), { description = "Switch to workspace 3" })
hl.bind("SUPER + code:13", hl.dsp.focus({ workspace = "4" }), { description = "Switch to workspace 4" })
hl.bind("SUPER + code:14", hl.dsp.focus({ workspace = "5" }), { description = "Switch to workspace 5" })
hl.bind("SUPER + code:15", hl.dsp.focus({ workspace = "6" }), { description = "Switch to workspace 6" })
hl.bind("SUPER + code:16", hl.dsp.focus({ workspace = "7" }), { description = "Switch to workspace 7" })
hl.bind("SUPER + code:17", hl.dsp.focus({ workspace = "8" }), { description = "Switch to workspace 8" })
hl.bind("SUPER + code:18", hl.dsp.focus({ workspace = "9" }), { description = "Switch to workspace 9" })
hl.bind("SUPER + code:19", hl.dsp.focus({ workspace = "10" }), { description = "Switch to workspace 10" })

hl.bind("SUPER + SHIFT + code:10", hl.dsp.window.move({ workspace = "1" }), { description = "Move window to workspace 1" })
hl.bind("SUPER + SHIFT + code:11", hl.dsp.window.move({ workspace = "2" }), { description = "Move window to workspace 2" })
hl.bind("SUPER + SHIFT + code:12", hl.dsp.window.move({ workspace = "3" }), { description = "Move window to workspace 3" })
hl.bind("SUPER + SHIFT + code:13", hl.dsp.window.move({ workspace = "4" }), { description = "Move window to workspace 4" })
hl.bind("SUPER + SHIFT + code:14", hl.dsp.window.move({ workspace = "5" }), { description = "Move window to workspace 5" })
hl.bind("SUPER + SHIFT + code:15", hl.dsp.window.move({ workspace = "6" }), { description = "Move window to workspace 6" })
hl.bind("SUPER + SHIFT + code:16", hl.dsp.window.move({ workspace = "7" }), { description = "Move window to workspace 7" })
hl.bind("SUPER + SHIFT + code:17", hl.dsp.window.move({ workspace = "8" }), { description = "Move window to workspace 8" })
hl.bind("SUPER + SHIFT + code:18", hl.dsp.window.move({ workspace = "9" }), { description = "Move window to workspace 9" })
hl.bind("SUPER + SHIFT + code:19", hl.dsp.window.move({ workspace = "10" }), { description = "Move window to workspace 10" })

hl.bind("SUPER + SHIFT + ALT + code:10", hl.dsp.window.move({ workspace = "1", follow = false }), { description = "Move window silently to workspace 1" })
hl.bind("SUPER + SHIFT + ALT + code:11", hl.dsp.window.move({ workspace = "2", follow = false }), { description = "Move window silently to workspace 2" })
hl.bind("SUPER + SHIFT + ALT + code:12", hl.dsp.window.move({ workspace = "3", follow = false }), { description = "Move window silently to workspace 3" })
hl.bind("SUPER + SHIFT + ALT + code:13", hl.dsp.window.move({ workspace = "4", follow = false }), { description = "Move window silently to workspace 4" })
hl.bind("SUPER + SHIFT + ALT + code:14", hl.dsp.window.move({ workspace = "5", follow = false }), { description = "Move window silently to workspace 5" })
hl.bind("SUPER + SHIFT + ALT + code:15", hl.dsp.window.move({ workspace = "6", follow = false }), { description = "Move window silently to workspace 6" })
hl.bind("SUPER + SHIFT + ALT + code:16", hl.dsp.window.move({ workspace = "7", follow = false }), { description = "Move window silently to workspace 7" })
hl.bind("SUPER + SHIFT + ALT + code:17", hl.dsp.window.move({ workspace = "8", follow = false }), { description = "Move window silently to workspace 8" })
hl.bind("SUPER + SHIFT + ALT + code:18", hl.dsp.window.move({ workspace = "9", follow = false }), { description = "Move window silently to workspace 9" })
hl.bind("SUPER + SHIFT + ALT + code:19", hl.dsp.window.move({ workspace = "10", follow = false }), { description = "Move window silently to workspace 10" })

hl.bind("SUPER + S", hl.dsp.workspace.toggle_special("scratchpad"), { description = "Toggle scratchpad" })
hl.bind("SUPER + ALT + S", hl.dsp.window.move({ workspace = "special:scratchpad", follow = false }), { description = "Move window to scratchpad" })

hl.bind("SUPER + TAB", hl.dsp.focus({ workspace = "e+1" }), { description = "Next workspace" })
hl.bind("SUPER + SHIFT + TAB", hl.dsp.focus({ workspace = "e-1" }), { description = "Previous workspace" })
hl.bind("SUPER + CTRL + TAB", hl.dsp.focus({ workspace = "previous" }), { description = "Former workspace" })

hl.bind("SUPER + SHIFT + ALT + LEFT", hl.dsp.workspace.move({ monitor = "l" }), { description = "Move workspace to left monitor" })
hl.bind("SUPER + SHIFT + ALT + RIGHT", hl.dsp.workspace.move({ monitor = "r" }), { description = "Move workspace to right monitor" })
hl.bind("SUPER + SHIFT + ALT + UP", hl.dsp.workspace.move({ monitor = "u" }), { description = "Move workspace to up monitor" })
hl.bind("SUPER + SHIFT + ALT + DOWN", hl.dsp.workspace.move({ monitor = "d" }), { description = "Move workspace to down monitor" })

hl.bind("SUPER + SHIFT + LEFT", hl.dsp.window.swap({ direction = "l" }), { description = "Swap window to the left" })
hl.bind("SUPER + SHIFT + RIGHT", hl.dsp.window.swap({ direction = "r" }), { description = "Swap window to the right" })
hl.bind("SUPER + SHIFT + UP", hl.dsp.window.swap({ direction = "u" }), { description = "Swap window up" })
hl.bind("SUPER + SHIFT + DOWN", hl.dsp.window.swap({ direction = "d" }), { description = "Swap window down" })

hl.bind("ALT + TAB", hl.dsp.window.cycle_next(), { description = "Focus on next window" })
hl.bind("ALT + SHIFT + TAB", hl.dsp.window.cycle_next({ next = false }), { description = "Focus on previous window" })
hl.bind("CTRL + ALT + TAB", hl.dsp.focus({ monitor = "+1" }), { description = "Focus on next monitor" })
hl.bind("CTRL + ALT + SHIFT + TAB", hl.dsp.focus({ monitor = "-1" }), { description = "Focus on previous monitor" })

hl.bind("SUPER + code:20", hl.dsp.window.resize({ x = -100, y = 0, relative = true }), { description = "Expand window left" })
hl.bind("SUPER + code:21", hl.dsp.window.resize({ x = 100, y = 0, relative = true }), { description = "Shrink window left" })
hl.bind("SUPER + SHIFT + code:20", hl.dsp.window.resize({ x = 0, y = -100, relative = true }), { description = "Shrink window up" })
hl.bind("SUPER + SHIFT + code:21", hl.dsp.window.resize({ x = 0, y = 100, relative = true }), { description = "Expand window down" })

hl.bind("SUPER + mouse_down", hl.dsp.focus({ workspace = "e+1" }), { description = "Scroll active workspace forward" })
hl.bind("SUPER + mouse_up", hl.dsp.focus({ workspace = "e-1" }), { description = "Scroll active workspace backward" })
hl.bind("SUPER + mouse:272", hl.dsp.window.drag(), { mouse = true, description = "Move window" })
hl.bind("SUPER + mouse:273", hl.dsp.window.resize(), { mouse = true, description = "Resize window" })

hl.bind("SUPER + G", hl.dsp.group.toggle(), { description = "Toggle window grouping" })
hl.bind("SUPER + ALT + G", hl.dsp.window.move({ out_of_group = true }), { description = "Move active window out of group" })
hl.bind("SUPER + ALT + LEFT", hl.dsp.window.move({ into_group = "l" }), { description = "Move window to group on left" })
hl.bind("SUPER + ALT + RIGHT", hl.dsp.window.move({ into_group = "r" }), { description = "Move window to group on right" })
hl.bind("SUPER + ALT + UP", hl.dsp.window.move({ into_group = "u" }), { description = "Move window to group on top" })
hl.bind("SUPER + ALT + DOWN", hl.dsp.window.move({ into_group = "d" }), { description = "Move window to group on bottom" })
hl.bind("SUPER + ALT + TAB", hl.dsp.group.next(), { description = "Next window in group" })
hl.bind("SUPER + ALT + SHIFT + TAB", hl.dsp.group.prev(), { description = "Previous window in group" })
hl.bind("SUPER + CTRL + LEFT", hl.dsp.group.prev(), { description = "Move grouped window focus left" })
hl.bind("SUPER + CTRL + RIGHT", hl.dsp.group.next(), { description = "Move grouped window focus right" })
hl.bind("SUPER + ALT + code:10", hl.dsp.group.active({ index = 1 }), { description = "Switch to group window 1" })
hl.bind("SUPER + ALT + code:11", hl.dsp.group.active({ index = 2 }), { description = "Switch to group window 2" })
hl.bind("SUPER + ALT + code:12", hl.dsp.group.active({ index = 3 }), { description = "Switch to group window 3" })
hl.bind("SUPER + ALT + code:13", hl.dsp.group.active({ index = 4 }), { description = "Switch to group window 4" })
hl.bind("SUPER + ALT + code:14", hl.dsp.group.active({ index = 5 }), { description = "Switch to group window 5" })
