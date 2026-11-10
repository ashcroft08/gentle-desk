# Measures one run of the built desktop app (release, real window) against a
# synthetic pi home, as an outside observer sees it: when the window appears,
# when the sidebar first exposes a row, when the list settles, the working set
# of the app and of its WebView2 processes, CPU seconds over idle windows and
# over scrolling, and whether the sidebar stays windowed while scrolling.
#
# UI Automation only, driven from the window element (never filtered by pid):
# the app's WebView2 exposes its DOM lazily, so every probe retries until its
# deadline and a single early miss is never read as absence.
#
#   pwsh -NoProfile -File tools/perf/measure-startup.ps1 `
#     -PiHome .verify/home50k -Label 50000 -OutJson .verify/run-50000.json
#
# The caller builds first (apps/desktop: npm ci; npm run build; then
# cargo build --release -p gentle-desk-app --features tauri/custom-protocol).
# The run uses a throw-away USERPROFILE/APPDATA/LOCALAPPDATA under -Scratch and
# the fake shell (GENTLE_SHELL_BIN) - never the real gentle-shell, never the
# user's own home or session cache.
param(
  [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path,
  [Parameter(Mandatory = $true)][string]$PiHome,
  [string]$Label = 'run',
  [string]$Scratch,
  [string]$OutJson,
  [int]$IdleSeconds = 10,
  [int]$ScrollSeconds = 10,
  [int]$SettleSeconds = 3,
  [int]$ScrollSteps = 40,
  [int]$MaxLoadSeconds = 900,
  [switch]$Warm
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

if (-not $Scratch) { $Scratch = Join-Path $RepoRoot ".verify\measure-$Label" }
$PiHome = (Resolve-Path $PiHome).Path
$exe = Join-Path $RepoRoot 'target\release\gentle-desk-app.exe'
if (-not (Test-Path $exe)) { throw "no release binary at $exe" }

# -Warm reuses the same scratch on purpose: the session index cache written by
# the previous run is then loaded, which is the returning user's case and the
# only way to reach a settled 50 000-chat list in seconds.
New-Item -ItemType Directory -Force -Path $Scratch,
  (Join-Path $Scratch 'userprofile'),
  (Join-Path $Scratch 'appdata'),
  (Join-Path $Scratch 'localappdata') | Out-Null
if ($Warm -and (Test-Path (Join-Path $Scratch 'localappdata\gentle-desk\session-index.json'))) {
  $Warm = $true
} elseif ($Warm) {
  throw "-Warm asked for but no session cache in $Scratch"
}

# The isolated run: the fake shell, and app data folders inside the scratch dir
# so the real config and the real session cache are never read or written.
$env:GENTLE_SHELL_BIN = Join-Path $RepoRoot 'tools\fake-gentle-shell\fake.mjs'
$env:USERPROFILE = Join-Path $Scratch 'userprofile'
$env:HOME = $env:USERPROFILE
$env:APPDATA = Join-Path $Scratch 'appdata'
$env:LOCALAPPDATA = Join-Path $Scratch 'localappdata'

$desktopEl = [System.Windows.Automation.AutomationElement]::RootElement
$AE = [System.Windows.Automation.AutomationElement]
$CT = [System.Windows.Automation.ControlType]
$scope = [System.Windows.Automation.TreeScope]

function Cond($property, $value) {
  New-Object System.Windows.Automation.PropertyCondition($property, $value)
}

function Find-All($rootEl, $condition) {
  try { return @($rootEl.FindAll($scope::Descendants, $condition)) } catch { return @() }
}

function Find-One($rootEl, $condition) {
  try { return $rootEl.FindFirst($scope::Descendants, $condition) } catch { return $null }
}

function Text-Of($el) {
  if (-not $el) { return $null }
  try { return $el.Current.Name } catch { return $null }
}

# The processes of this run: the app (the window's own process, not the pid
# Start-Process reports) and every descendant, found by walking the parent
# chain, which is where the WebView2 browser, gpu, network and renderer
# processes hang. The machine runs other WebView2 hosts (the shell's own
# widgets), so nothing is summed by image name.
function Get-TreePids([int]$parentPid) {
  $found = New-Object System.Collections.Generic.List[int]
  $found.Add($parentPid)
  $queue = New-Object System.Collections.Generic.Queue[int]
  $queue.Enqueue($parentPid)
  $all = Get-CimInstance Win32_Process
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

function Get-TreeUsage([int[]]$pids) {
  $cpu = 0.0
  $ws = 0L
  $parts = @()
  foreach ($id in $pids) {
    $p = Get-Process -Id $id -ErrorAction SilentlyContinue
    if (-not $p) { continue }
    $cpu += $p.TotalProcessorTime.TotalSeconds
    $ws += $p.WorkingSet64
    $parts += [ordered]@{
      pid = $id
      name = $p.ProcessName
      cpu_s = [math]::Round($p.TotalProcessorTime.TotalSeconds, 3)
      ws_mb = [math]::Round($p.WorkingSet64 / 1MB, 1)
      threads = $p.Threads.Count
    }
  }
  return [ordered]@{
    cpu_s = [math]::Round($cpu, 3)
    ws_mb = [math]::Round($ws / 1MB, 1)
    parts = $parts
  }
}

function Stop-Tree([int[]]$pids) {
  foreach ($id in $pids) { Stop-Process -Id $id -Force -ErrorAction SilentlyContinue }
}

# The session index cache this run writes (30 s throttle, flushed on the way
# out): its timestamp and size say whether a CPU spike belongs to a cache write
# instead of to the list.
function Cache-Fingerprint {
  $path = Join-Path $env:LOCALAPPDATA 'gentle-desk\session-index.json'
  if (-not (Test-Path $path)) { return $null }
  $item = Get-Item $path
  return [ordered]@{
    mtime = $item.LastWriteTime.ToString('HH:mm:ss.fff')
    mb = [math]::Round($item.Length / 1MB, 1)
  }
}

$sessionCount = @(Get-ChildItem -Path (Join-Path $PiHome 'sessions') -Recurse -Filter *.jsonl -ErrorAction SilentlyContinue).Count
$result = [ordered]@{
  label = $Label
  pi_home = $PiHome
  sessions = $sessionCount
  started_at = (Get-Date).ToString('o')
}

$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
$app = Start-Process -FilePath $exe -ArgumentList @('--home', $PiHome) -PassThru
$launcherPid = $app.Id
$result.launcher_pid = $launcherPid

# Phase 1: the window, the first row and the settled list. The working set is
# sampled through the load because the peak is what a 50 000-chat first pass
# costs, not the value it happens to end on.
$winCond = Cond $AE::NameProperty 'gentle-desk'
$itemCond = Cond $AE::ControlTypeProperty $CT::TreeItem
$textCond = Cond $AE::ControlTypeProperty $CT::Text

$win = $null
$appPid = $null
$windowMs = $null
$firstRowMs = $null
$firstTextMs = $null
$settledMs = $null
$peakAppWs = 0L
$peakTreeWs = 0L
$lastRowCount = 0
$stableSince = $null
$pids = @()
$progress = @()
$nextProbeAt = 0
# Probe often while the boot is short (a warm cache settles in well under a
# second), then back off: the tree walk forces the webview's accessibility tree
# and is work of its own while a cold 50 000-chat pass runs for minutes.
function Probe-Delay([int64]$elapsedMs) {
  if ($elapsedMs -lt 3000) { return 50 }
  if ($elapsedMs -lt 20000) { return 1000 }
  return 5000
}
$deadline = $stopwatch.ElapsedMilliseconds + (1000 * $MaxLoadSeconds)
while ($stopwatch.ElapsedMilliseconds -lt $deadline) {
  if (-not $win) {
    $win = $desktopEl.FindFirst($scope::Children, $winCond)
    if ($win) {
      $windowMs = $stopwatch.ElapsedMilliseconds
      $appPid = $win.Current.ProcessId
      $pids = Get-TreePids $appPid
    }
  }
  if (-not $win) { Start-Sleep -Milliseconds 20; continue }
  # Every probe below costs the webview work of its own (UIA forces its
  # accessibility tree, Get-Process reads the tree), so all of it is on the
  # schedule Probe-Delay sets, never on the 25 ms poll of the window itself.
  if ($stopwatch.ElapsedMilliseconds -lt $nextProbeAt) { Start-Sleep -Milliseconds 25; continue }
  $nextProbeAt = $stopwatch.ElapsedMilliseconds + (Probe-Delay $stopwatch.ElapsedMilliseconds)
  if (-not $firstRowMs) {
    if (Find-One $win $itemCond) { $firstRowMs = $stopwatch.ElapsedMilliseconds }
  }
  if (-not $firstTextMs) {
    if ((Find-All $win $textCond).Count -gt 0) { $firstTextMs = $stopwatch.ElapsedMilliseconds }
  }
  $usage = Get-TreeUsage $pids
  if ($usage.ws_mb -gt $peakTreeWs) { $peakTreeWs = $usage.ws_mb }
  $appWs = 0L
  foreach ($part in $usage.parts) { if ($part.pid -eq $appPid) { $appWs = [int64]($part.ws_mb * 1MB) } }
  if ($appWs -gt $peakAppWs) { $peakAppWs = $appWs }
  $rows = Find-All $win $itemCond
  $reading = $null
  foreach ($t in (Find-All $win $textCond)) {
    $n = Text-Of $t
    if ($n -and $n -like 'Reading chats*') { $reading = $n; break }
  }
  if ($reading) { $progress += [ordered]@{ ms = $stopwatch.ElapsedMilliseconds; line = $reading } }
  if ($rows.Count -ne $lastRowCount) {
    $lastRowCount = $rows.Count
    $stableSince = $stopwatch.ElapsedMilliseconds
  }
  $stableFor = if ($stableSince) { $stopwatch.ElapsedMilliseconds - $stableSince } else { 0 }
  if (-not $settledMs -and -not $reading -and $rows.Count -gt 0 -and $stableFor -ge 700) {
    $settledMs = $stopwatch.ElapsedMilliseconds
    break
  }
  Start-Sleep -Milliseconds 25
}
if (-not $win -or -not $settledMs) {
  $result.error = 'the window or the settled list never appeared'
  if ($pids.Count) { Stop-Tree $pids }
  $result | ConvertTo-Json -Depth 6
  exit 1
}

$result.app_pid = $appPid
$result.window_ms = $windowMs
$result.first_row_ms = $firstRowMs
$result.first_text_ms = $firstTextMs
$result.settled_ms = $settledMs
$result.load_peak_app_ws_mb = [math]::Round($peakAppWs / 1MB, 1)
$result.load_peak_tree_ws_mb = $peakTreeWs
$result.rows_at_settle = $lastRowCount
$result.load_progress = $progress
$result.processes_at_settle = (Get-TreeUsage $pids)

# Let the deferred rendering of the settle land, then measure the idle windows
# with no UI Automation call at all: an accessibility query is work of its own
# and would be counted as the app's. The process tree is read again here: the
# WebView2 processes are children that only exist once the webview boots.
Start-Sleep -Seconds $SettleSeconds
$pids = Get-TreePids $appPid
$result.processes_idle = (Get-TreeUsage $pids)
$cacheSamples = @()
$cacheSamples += [ordered]@{ at = 'settled'; fingerprint = (Cache-Fingerprint) }
$idle = @()
$previous = Get-TreeUsage $pids
foreach ($round in 1..2) {
  Start-Sleep -Seconds $IdleSeconds
  $now = Get-TreeUsage $pids
  $cacheSamples += [ordered]@{ at = "idle$round"; fingerprint = (Cache-Fingerprint) }
  $appDelta = 0.0
  $webviewDelta = 0.0
  foreach ($part in $now.parts) {
    $before = $previous.parts | Where-Object { $_.pid -eq $part.pid } | Select-Object -First 1
    $delta = $part.cpu_s - ($(if ($before) { $before.cpu_s } else { 0.0 }))
    if ($part.pid -eq $appPid) { $appDelta += $delta } else { $webviewDelta += $delta }
  }
  $idle += [ordered]@{
    round = $round
    seconds = $IdleSeconds
    cpu_s = [math]::Round($now.cpu_s - $previous.cpu_s, 3)
    app_cpu_s = [math]::Round($appDelta, 3)
    webview_cpu_s = [math]::Round($webviewDelta, 3)
    ws_mb = $now.ws_mb
  }
  $previous = $now
}
$result.idle = $idle
$result.idle_ws_mb = $idle[-1].ws_mb
$result.cache = @($cacheSamples)
$result.uia_elements_settle = (Find-All $win ([System.Windows.Automation.Condition]::TrueCondition)).Count

# Phase 3: scroll the sidebar. The path is the UIA ScrollPattern of the scroll
# container above the tree element when the webview exposes one; otherwise a
# real mouse wheel over the sidebar, which is the user's own path.
$treeEl = Find-One $win (Cond $AE::ControlTypeProperty $CT::Tree)
$result.sidebar_tree_exposed = [bool]$treeEl
$scrollEl = $null
$scrollPattern = $null
$cursor = $treeEl
$walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
while ($cursor) {
  try {
    $scrollPattern = $cursor.GetCurrentPattern([System.Windows.Automation.ScrollPattern]::Pattern)
    if ($scrollPattern) { $scrollEl = $cursor; break }
  } catch { $scrollPattern = $null }
  try { $cursor = $walker.GetParent($cursor) } catch { $cursor = $null }
}
$result.scroll_owner = if ($scrollEl) { "$($scrollEl.Current.ControlType.ProgrammaticName)|$($scrollEl.Current.Name)" } else { $null }

Add-Type -Namespace Win32 -Name Cursor -MemberDefinition @'
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, int d, System.UIntPtr e);
'@

$samples = @()
$scrollCpuBefore = Get-TreeUsage $pids
$scrollSw = [System.Diagnostics.Stopwatch]::StartNew()
if ($scrollPattern) {
  for ($i = 1; $i -le $ScrollSteps; $i++) {
    $percent = [int](100 * $i / $ScrollSteps)
    try { $scrollPattern.SetScrollPercent(-1, $percent) } catch { }
    Start-Sleep -Milliseconds ([math]::Max(50, [int](1000 * $ScrollSeconds / $ScrollSteps)))
    $rowsNow = Find-All $win $itemCond
    $names = @()
    foreach ($r in ($rowsNow | Select-Object -First 3)) { $n = Text-Of $r; if ($n) { $names += $n } }
    $samples += [ordered]@{ percent = $percent; items = $rowsNow.Count; first = $names }
  }
} else {
  $rect = $win.Current.BoundingRectangle
  $x = [int]($rect.Left + 120)
  $y = [int]($rect.Top + $rect.Height / 2)
  [Win32.Cursor]::SetCursorPos($x, $y) | Out-Null
  $ticks = [math]::Max(1, [int](1000 * $ScrollSeconds / $ScrollSteps))
  for ($i = 1; $i -le $ScrollSteps; $i++) {
    [Win32.Cursor]::mouse_event(0x0800, 0, 0, -120, [System.UIntPtr]::Zero)
    Start-Sleep -Milliseconds $ticks
    $rowsNow = Find-All $win $itemCond
    $names = @()
    foreach ($r in ($rowsNow | Select-Object -First 3)) { $n = Text-Of $r; if ($n) { $names += $n } }
    $samples += [ordered]@{ notch = $i; items = $rowsNow.Count; first = $names }
  }
}
$scrollSw.Stop()
$scrollCpuAfter = Get-TreeUsage $pids
$result.scroll_mode = if ($scrollEl) { 'uia-scrollpattern' } else { 'mouse-wheel' }
$result.scroll_seconds = [math]::Round($scrollSw.Elapsed.TotalSeconds, 2)
$result.scroll_cpu_s = [math]::Round($scrollCpuAfter.cpu_s - $scrollCpuBefore.cpu_s, 3)
$counts = @($samples | ForEach-Object { $_.items })
$result.scroll_items_min = ($counts | Measure-Object -Minimum).Minimum
$result.scroll_items_max = ($counts | Measure-Object -Maximum).Maximum
$result.scroll_samples = $samples
$result.uia_elements_after_scroll = (Find-All $win ([System.Windows.Automation.Condition]::TrueCondition)).Count
$result.ws_after_scroll_mb = $scrollCpuAfter.ws_mb
$result.reading_line_after_scroll = $null
foreach ($t in (Find-All $win $textCond)) {
  $n = Text-Of $t
  if ($n -and $n -like 'Reading chats*') { $result.reading_line_after_scroll = $n; break }
}
$result.window_name = Text-Of $win
$result.window_class = try { $win.Current.ClassName } catch { $null }
$result.cache_after_scroll = Cache-Fingerprint

Stop-Tree $pids

$json = $result | ConvertTo-Json -Depth 8
if ($OutJson) { $json | Out-File -FilePath $OutJson -Encoding utf8 }
$json
