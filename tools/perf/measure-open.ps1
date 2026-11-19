# Measures how long one chat takes to show CONTENT after it is opened (the
# perceived open of `chat-open-latency.md` S1), in the built desktop app's real
# window, as an outside observer (UI Automation) sees it:
#
#   open_ms              : from the double-click on the sidebar row to the FIRST
#                          text the window exposes that was not there before the
#                          click, i.e. the thread answered at all. `first_text_ms`
#                          repeats the same observation under its own name.
#   first_content_row_ms : to the first transcript row, a new text that is NOT the
#                          opening line ("Opening chat") - content, not just the
#                          status line. This is the number S1 is about.
#   painted_ms           : to that row count stopping to grow (the file pre-paint
#                          is done). It includes the settle window itself
#                          (-SettleSeconds, 1 s by default), so it is the last
#                          row plus the wait that proved it was the last.
#   empty_frame_before_content : whether an opening-only frame (text but no
#                          transcript row) was exposed before the first content
#                          row, the frame T2 was about.
#
#   pwsh -NoProfile -File tools/perf/measure-open.ps1 `
#     -PiHome .verify/home-open -Session 'chat 0' `
#     -OutJson .verify/open-1.json
#
#   node tools/perf/gen-home.mjs .verify/home-open 5 130   # the synthetic home
#
# The caller builds first (apps/desktop: npm ci; npm run build; then
# cargo build --release -p gentle-desk-app --features tauri/custom-protocol).
# Nothing is built here. The run uses the real app against the home you give it:
# like measure-startup.ps1 it never touches YOUR real home unless you point
# -PiHome at it. The chat must already exist in that home; the script opens the
# FIRST chat row whose name contains -Session (or the first chat row when
# omitted). Run it several times; the JSON carries every number, no aggregation.
# Reuse a -Scratch across runs on purpose: the session index cache written by the
# first run is then loaded (the returning user's case).
#
# Four things the window needs, each learned the hard way on this machine:
#
# 1. The binary is `target\release\gentle-desk-app.exe`. `gentle-desk.exe` is a
#    stale build of another package name and renders a window with no webview
#    content - a harness that launches it probes a window that never has an
#    accessibility tree at all.
# 2. The window is found by PROCESS, never by title: the app owns several
#    top-level windows, the first of which is an invisible winit message window
#    titled `Tao Thread Event Target` that exposes nothing. The real one has
#    class `Tauri Window` and title `Gentle Shell Desktop` (it is NOT
#    `- gentle-desk`; titles ending in `gentle-desk` on this machine belong to
#    Windows Terminal and Brave).
# 3. The config MUST hold `shell_home`. The app asks the home question when pi is
#    found, no choice is stored and the launcher resolves
#    (`bridge/shell.rs`, `ask`), and that welcome screen is what the window holds
#    instead of the sidebar.
# 4. The config must NOT name a workspace whose folder is missing: the sidebar
#    then draws its header, its chats never appear, and the run has nothing to
#    open. Leaving `workspaces` empty lets the home's own sessions build the
#    list, which is exactly what the fake shell and a synthetic home need.

param(
  [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path,
  [Parameter(Mandatory = $true)][string]$PiHome,
  [string]$Session = '',
  [string]$OutJson,
  [string]$Scratch = (Join-Path ([System.IO.Path]::GetTempPath()) ('gentle-desk-open-' + [guid]::NewGuid().ToString('N').Substring(0,8))),
  [int]$SettleSeconds = 1,
  [int]$MaxOpenSeconds = 60,
  [int]$RowWaitSeconds = 90
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -Namespace Win -Name Native -MemberDefinition @'
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int code);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint flags, int dx, int dy, uint data, System.UIntPtr extra);
'@
$AE = [System.Windows.Automation.AutomationElement]
$CT = [System.Windows.Automation.ControlType]
$scope = [System.Windows.Automation.TreeScope]::Descendants
$any = [System.Windows.Automation.Condition]::TrueCondition

function Cond($prop, $val) { New-Object System.Windows.Automation.PropertyCondition($prop, $val) }
$itemCond = Cond $AE::ControlTypeProperty $CT::TreeItem
$textCond = Cond $AE::ControlTypeProperty $CT::Text

function Find-All($root, $cond) {
  try { return @($root.FindAll($scope, $cond)) } catch { return @() }
}

function Text-Of($el) {
  if (-not $el) { return $null }
  try { return $el.Current.Name } catch { return $null }
}

# The app binary of this repo. `gentle-desk.exe` is a stale build of an older
# package name; the app is `gentle-desk-app.exe`.
$exe = Join-Path $RepoRoot 'target\release\gentle-desk-app.exe'
if (-not (Test-Path $exe)) { Write-Error "build first: $exe missing"; exit 1 }

# Leftovers of an earlier failed run keep their own (empty) windows and would be
# picked up by the process scan, so they are stopped first. Only processes whose
# own image is this repo's release binary qualify - never another app's.
Get-CimInstance Win32_Process |
  Where-Object { $_.ExecutablePath -eq $exe } |
  ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
Start-Sleep -Milliseconds 300

# The hermetic run: the FAKE shell (the real gentle-shell spawns per chat and
# would measure its boot, not the app's), and a throw-away USERPROFILE/APPDATA/
# LOCALAPPDATA under -Scratch so the real config and session cache are never
# read or written.
New-Item -ItemType Directory -Force -Path (Join-Path $Scratch 'userprofile'),
  (Join-Path $Scratch 'appdata'), (Join-Path $Scratch 'localappdata') | Out-Null
$env:GENTLE_SHELL_BIN = Join-Path $RepoRoot 'tools\fake-gentle-shell\fake.mjs'
$env:USERPROFILE = Join-Path $Scratch 'userprofile'
$env:HOME = $env:USERPROFILE
$env:APPDATA = Join-Path $Scratch 'appdata'
$env:LOCALAPPDATA = Join-Path $Scratch 'localappdata'

# The config this run reads: the home question already answered (`shell_home`),
# and no workspace of its own, so the sidebar lists what the home's sessions say
# (a config workspace whose folder is missing would hide its chats).
New-Item -ItemType Directory -Force -Path (Join-Path $env:APPDATA 'gentle-desk') | Out-Null
@{ version = 1; theme = 'system'; shell_home = 'isolated'; workspaces = @() } |
  ConvertTo-Json -Depth 4 | Set-Content -Encoding UTF8 (Join-Path $env:APPDATA 'gentle-desk\config.json')

$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
$app = Start-Process -FilePath $exe -ArgumentList @('--home', $PiHome) -PassThru

# The processes of this run: the app and every descendant (the WebView2 browser,
# gpu, network and renderer processes hang off it), found by walking the parent
# chain so no other WebView2 host on the machine is ever touched.
function Get-TreePids([int]$rootPid) {
  $all = Get-CimInstance Win32_Process
  $found = New-Object System.Collections.Generic.List[int]
  $found.Add($rootPid)
  $queue = New-Object System.Collections.Generic.Queue[int]
  $queue.Enqueue($rootPid)
  while ($queue.Count -gt 0) {
    $current = $queue.Dequeue()
    foreach ($kid in ($all | Where-Object { $_.ParentProcessId -eq $current })) {
      if ($found.Contains([int]$kid.ProcessId)) { continue }
      $found.Add([int]$kid.ProcessId)
      $queue.Enqueue([int]$kid.ProcessId)
    }
  }
  return $found
}

# The window, by process and class: the app's first top-level window is the
# invisible winit message window and exposes no accessibility tree at all.
$desktopEl = $AE::RootElement
$win = $null
$windowMs = $null
$windowDeadline = $stopwatch.ElapsedMilliseconds + 30000
while ($stopwatch.ElapsedMilliseconds -lt $windowDeadline -and -not $win) {
  foreach ($top in @($desktopEl.FindAll([System.Windows.Automation.TreeScope]::Children, $any))) {
    $topPid = 0
    $class = ''
    try { $topPid = $top.Current.ProcessId; $class = $top.Current.ClassName } catch { continue }
    if ($topPid -eq $app.Id -and $class -eq 'Tauri Window') { $win = $top; break }
  }
  if (-not $win) { Start-Sleep -Milliseconds 100 }
}
if (-not $win) {
  Write-Error "the app's window (class 'Tauri Window') never appeared for pid $($app.Id)"
  exit 1
}
$windowMs = $stopwatch.ElapsedMilliseconds

# The window has to be visible and in front while the webview boots: a window
# left behind another one still paints its panes but its DOM (and so its whole
# accessibility tree) never shows up, which is the "window found, zero elements"
# run. It is brought forward once here and again on every retry below.
$handle = [IntPtr]$win.Current.NativeWindowHandle
[void][Win.Native]::ShowWindow($handle, 9)
[void][Win.Native]::SetForegroundWindow($handle)

# The webview exposes its accessibility tree lazily: the window shows its panes
# first and its Document (the DOM) a moment later, so every probe retries until
# its deadline and a single early miss is never read as absence.
$docCond = Cond $AE::ControlTypeProperty $CT::Document
$docDeadline = $stopwatch.ElapsedMilliseconds + 30000
while ($stopwatch.ElapsedMilliseconds -lt $docDeadline -and -not (Find-All $win $docCond).Count) {
  [void][Win.Native]::SetForegroundWindow($handle)
  Start-Sleep -Milliseconds 150
}

# The chat rows. Every sidebar TreeItem offers ExpandCollapsePattern and
# InvokePattern alike, headers and chats both, so the rows are told apart by the
# workspace they belong to: a header carries its workspace's label, spelled
# exactly as the "New chat in <label>" button beside it, and a chat row does not
# start with it. With no header matched at all the first row is skipped instead,
# because a tree whose first row is a chat is not a shape this app draws.
function Get-Chat-Rows($window, $condition, $labels) {
  $rows = Find-All $window $condition
  $chats = @()
  $headers = 0
  foreach ($row in $rows) {
    $name = Text-Of $row
    if (-not $name) { continue }
    $isHeader = $false
    foreach ($label in $labels) {
      if ($label -and ($name -eq $label -or $name.StartsWith($label + ' '))) { $isHeader = $true; break }
    }
    if ($isHeader) { $headers++; continue }
    $chats += $row
  }
  if ($headers -eq 0 -and $rows.Count -gt 1) { return @($rows | Select-Object -Skip 1) }
  return $chats
}

# The workspace labels, read off the per-workspace "New chat in <label>" button.
function Get-Workspace-Labels($window) {
  $labels = @()
  $buttonCond = Cond $AE::ControlTypeProperty $CT::Button
  foreach ($button in (Find-All $window $buttonCond)) {
    $name = Text-Of $button
    if ($name -and $name.StartsWith('New chat in ')) { $labels += $name.Substring('New chat in '.Length).Trim() }
  }
  return $labels
}

$rows = @()
$labelDeadline = $stopwatch.ElapsedMilliseconds + (1000 * $RowWaitSeconds)
while ($stopwatch.ElapsedMilliseconds -lt $labelDeadline -and $rows.Count -eq 0) {
  $rows = Get-Chat-Rows $win $itemCond (Get-Workspace-Labels $win)
  if ($rows.Count -eq 0) {
    [void][Win.Native]::SetForegroundWindow($handle)
    Start-Sleep -Milliseconds 250
  }
}
$rowsMs = $stopwatch.ElapsedMilliseconds
if ($rows.Count -eq 0) {
  $result = [ordered]@{
    label = 'chat-open'
    pi_home = $PiHome
    session = $null
    open_ms = $null
    first_text_ms = $null
    first_content_row_ms = $null
    painted_ms = $null
    empty_frame_before_content = $null
    rows_at_paint = 0
    window_ms = $windowMs
    error = "no chat row appeared in $RowWaitSeconds s"
    measured_at = (Get-Date).ToString('o')
  }
  foreach ($id in (Get-TreePids $app.Id)) { Stop-Process -Id $id -Force -ErrorAction SilentlyContinue }
  $out = if ($OutJson) { $OutJson } else { '.verify/open.json' }
  New-Item -ItemType Directory -Force -Path (Split-Path $out -Parent) | Out-Null
  $result | ConvertTo-Json -Depth 4 | Set-Content -Encoding UTF8 $out
  $result | ConvertTo-Json -Depth 4
  exit 1
}

# The row to open: the first chat row whose name contains -Session, else the first.
# Its name is read now, before the click: the tree re-renders when the chat opens
# and the old element then answers nothing.
$target = $null
foreach ($row in $rows) {
  $name = Text-Of $row
  if ($name -and ($Session -eq '' -or $name.Contains($Session))) { $target = $row; break }
}
if (-not $target) { $target = $rows[0] }
$targetName = Text-Of $target
$rowNames = @($rows | ForEach-Object { Text-Of $_ } | Where-Object { $_ })

# What the window already said, so the thread's own first frame is told apart
# from the sidebar's labels: a text is NEW when it was not there before the click.
$before = New-Object 'System.Collections.Generic.HashSet[string]'
foreach ($text in (Find-All $win $textCond)) {
  $name = Text-Of $text
  if ($name -and $name.Trim()) { [void]$before.Add($name.Trim()) }
}

# Open it. A TreeItem answers SelectionItem, not Invoke, so the row is clicked
# for real: the window is brought forward first, or the click would land on
# whatever covers it. SetCursorPos plus a relative mouse_event - the absolute
# form needs screen-space flags and would jump the pointer to a screen corner.
[void][Win.Native]::ShowWindow($handle, 9)
[void][Win.Native]::SetForegroundWindow($handle)
Start-Sleep -Milliseconds 120
$invoke = $null
if ($target.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern, [ref]$invoke)) {
  $openMs = [int]$stopwatch.ElapsedMilliseconds
  $invoke.Invoke()
} else {
  $rect = $target.Current.BoundingRectangle
  $cx = [int]($rect.X + $rect.Width / 2)
  $cy = [int]($rect.Y + $rect.Height / 2)
  [void][Win.Native]::SetCursorPos($cx, $cy)
  Start-Sleep -Milliseconds 60
  $openMs = [int]$stopwatch.ElapsedMilliseconds
  [Win.Native]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero)
  [Win.Native]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero)
  Start-Sleep -Milliseconds 120
  [Win.Native]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero)
  [Win.Native]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero)
}

# Watch the window for its first new text, its first content row and its settled
# paint. The opening line is text, not content: "Opening chat"/"Abriendo el chat"
# is the status the thread shows while the child has not answered yet.
$openingRe = '^(Opening chat|Abriendo el chat)$'
$firstTextMs = $null
$contentMs = $null
$paintedMs = $null
$emptySeen = $false
$openingSeen = $false
$lastCount = 0
$stableSince = $null
$watchDeadline = $openMs + (1000 * $MaxOpenSeconds)
$nextProbeAt = $openMs
while ($stopwatch.ElapsedMilliseconds -lt $watchDeadline) {
  if ($stopwatch.ElapsedMilliseconds -lt $nextProbeAt) { Start-Sleep -Milliseconds 25; continue }
  # Every UIA walk costs the webview work of its own, so the probes are on a
  # fixed cadence instead of one per loop pass.
  $nextProbeAt = $stopwatch.ElapsedMilliseconds + 100
  $fresh = @()
  foreach ($text in (Find-All $win $textCond)) {
    $name = Text-Of $text
    if (-not $name -or -not $name.Trim()) { continue }
    $trimmed = $name.Trim()
    if (-not $before.Contains($trimmed)) { $fresh += $trimmed }
  }
  $elapsed = [int]$stopwatch.ElapsedMilliseconds - $openMs
  if ($fresh.Count -gt 0 -and -not $firstTextMs) { $firstTextMs = $elapsed }
  $content = @($fresh | Where-Object { $_ -notmatch $openingRe })
  if ($content.Count -gt 0 -and -not $contentMs) { $contentMs = $elapsed }
  if ($fresh.Count -gt 0 -and $content.Count -eq 0) {
    $openingSeen = $true
    if (-not $contentMs) { $emptySeen = $true }
  }
  if ($content.Count -ne $lastCount) {
    $lastCount = $content.Count
    $stableSince = $stopwatch.ElapsedMilliseconds
  }
  $stableFor = if ($stableSince) { $stopwatch.ElapsedMilliseconds - $stableSince } else { 0 }
  if ($content.Count -gt 0 -and $stableFor -ge (1000 * $SettleSeconds)) {
    $paintedMs = $elapsed
    break
  }
  Start-Sleep -Milliseconds 25
}

foreach ($id in (Get-TreePids $app.Id)) { Stop-Process -Id $id -Force -ErrorAction SilentlyContinue }

$result = [ordered]@{
  label = 'chat-open'
  pi_home = $PiHome
  session = $targetName
  open_ms = $firstTextMs
  first_text_ms = $firstTextMs
  first_content_row_ms = $contentMs
  painted_ms = $paintedMs
  empty_frame_before_content = $emptySeen
  opening_line_seen = $openingSeen
  rows_at_paint = $lastCount
  settle_seconds = $SettleSeconds
  sidebar_rows = $rows.Count
  sidebar_row_names = @($rowNames | Select-Object -First 12)
  window_ms = $windowMs
  sidebar_rows_ms = $rowsMs
  measured_at = (Get-Date).ToString('o')
}
if (-not $firstTextMs) { $result.error = "no new text appeared in $MaxOpenSeconds s after the click" }
$out = if ($OutJson) { $OutJson } else { '.verify/open.json' }
New-Item -ItemType Directory -Force -Path (Split-Path $out -Parent) | Out-Null
$result | ConvertTo-Json -Depth 4 | Set-Content -Encoding UTF8 $out
$result | ConvertTo-Json -Depth 4
if (-not $firstTextMs) { exit 1 }
