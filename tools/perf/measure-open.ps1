# Measures how long one chat takes to show CONTENT after it is opened (the
# perceived open of `chat-open-latency.md` S1), in the built desktop app's real
# window, as an outside observer (UI Automation) sees it:
#
#   open_ms   : from the double-click on the sidebar row to the FIRST text the
#               thread's own scrollable area exposes
#   rows_ms   : to the first transcript row (a Text that is NOT the opening
#               line), i.e. content, not just the "Opening chat" line
#   paint_ms  : to the transcript's LAST row being exposed (the file pre-paint
#               is done), with the same settle rule the sidebar uses
#   empty_ms  : rows the thread exposed with no transcript row present, the
#               number T2 was about (an empty frame before content)
#
#   pwsh -NoProfile -File tools/perf/measure-open.ps1 `
#     -PiHome .verify/home-open -Session 'some-session-name' `
#     -OutJson .verify/open-1.json
#
# The caller builds first (apps/desktop: npm ci; npm run build; then
# cargo build --release -p gentle-desk-app --features tauri/custom-protocol).
# The run uses the real app against the home you give it: like
# measure-startup.ps1 it never touches YOUR real home unless you point
# -PiHome at it. The chat must already exist in that home; the script picks
# the FIRST sidebar row whose name contains -Session (or row 1 when omitted).
# Run it several times; the JSON carries every number, no aggregation.

param(
  [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path,
  [Parameter(Mandatory = $true)][string]$PiHome,
  [string]$Session = '',
  [string]$OutJson,
  [int]$SettleSeconds = 3,
  [int]$MaxOpenSeconds = 60
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$AE  = [System.Windows.Automation.AutomationElement]
$CT  = [System.Windows.Automation.ControlType]
$scope = [System.Windows.Automation.TreeScope]::Children
$Cond  = { param($p) New-Object System.Windows.Automation.PropertyCondition($p[0], $p[1]) }
function Cond($prop, $val) { New-Object System.Windows.Automation.PropertyCondition($prop, $val) }
$winCond  = Cond $AE::NameProperty 'gentle-desk'
$itemCond = Cond $AE::ControlTypeProperty $CT::TreeItem
$textCond = Cond $AE::ControlTypeProperty $CT::Text
$docCond  = Cond $AE::ControlTypeProperty $CT::Document

function Text-Of($el) {
  try { return $el.Current.Name } catch { return $null }
}

function Find-One($root, $cond) { $root.FindFirst($scope, $cond) }
function Find-All($root, $cond) { @($root.FindAll($scope, $cond)) }

$exe = Join-Path $RepoRoot 'target\release\gentle-desk.exe'
if (-not (Test-Path $exe)) { Write-Error "build first: $exe missing"; exit 1 }

$desktopEl = $AE::RootElement
$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
$app = Start-Process -FilePath $exe -ArgumentList @('--home', $PiHome) -PassThru
$deadline = $stopwatch.ElapsedMilliseconds + 30000
$win = $null
while ($stopwatch.ElapsedMilliseconds -lt $deadline -and -not $win) {
  $win = $desktopEl.FindFirst($scope, $winCond)
  if (-not $win) { Start-Sleep -Milliseconds 50 }
}
if (-not $win) { Write-Error 'the window never appeared'; exit 1 }
Start-Sleep -Seconds 2   # let the sidebar settle before the measured click

# The row to open: the first TreeItem whose name contains -Session, else row 1.
$rows = Find-All $win $itemCond
$target = $null
foreach ($row in $rows) {
  $name = $row.Current.Name
  if ($name -and ($Session -eq '' -or $name.Contains($Session))) { $target = $row; break }
}
if (-not $target -and $rows.Count -gt 0) { $target = $rows[0] }
if (-not $target) { Write-Error 'no sidebar row to open'; exit 1 }

$invoke = $null
if ($target.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern.Identifier, [ref]$invoke)) { $invoke.Invoke() }
else {
  $rect = $target.Current.BoundingRectangle
  $cx = [int]($rect.X + $rect.Width / 2); $cy = [int]($rect.Y + $rect.Height / 2)
  # A real double click on the row's centre: the app opens on a click of its own.
  Add-Type -Namespace Win -Name Mouse -MemberDefinition '[DllImport("user32.dll")] public static extern void mouse_event(uint f, uint x, uint y, uint d, UIntPtr e);'
  [Win.Mouse]::mouse_event(2, $cx, $cy, 0, [UIntPtr]::Zero)
  [Win.Mouse]::mouse_event(4, $cx, $cy, 0, [UIntPtr]::Zero)
  Start-Sleep -Milliseconds 120
  [Win.Mouse]::mouse_event(2, $cx, $cy, 0, [UIntPtr]::Zero)
  [Win.Mouse]::mouse_event(4, $cx, $cy, 0, [UIntPtr]::Zero)
}
$openMs = $stopwatch.ElapsedMilliseconds

# Watch the thread area for its first text, its first content row and its
# settled paint. The opening line ("Opening chat") is text, not content: a
# transcript row is any Text whose name is longer than the line can be.
$openingRe = '^(Opening|Abriendo|Trabajando|working)'
$contentMs = $null
$paintedMs = $null
$emptyFrames = 0
$emptySeen = $false
$lastCount = 0
$stableSince = $null
$watchDeadline = $openMs + (1000 * $MaxOpenSeconds)
$lastProbe = 0
while ($stopwatch.ElapsedMilliseconds -lt $watchDeadline) {
  if ($stopwatch.ElapsedMilliseconds -lt $lastProbe) { Start-Sleep -Milliseconds 25; continue }
  $lastProbe = $stopwatch.ElapsedMilliseconds + 100   # UIA probes cost the webview: bounded cadence
  $doc = Find-One $win $docCond
  if (-not $doc) { Start-Sleep -Milliseconds 100; continue }
  $texts = Find-All $doc $textCond
  $names = foreach ($t in $texts) { Text-Of $t }
  $nonEmpty = @($names | Where-Object { $_ -and $_.Trim().Length -gt 0 })
  if ($nonEmpty.Count -gt 0 -and -not $contentMs) { $contentMs = $stopwatch.ElapsedMilliseconds - $openMs }
  $rowsNow = @($nonEmpty | Where-Object { $_ -notmatch $openingRe })
  if ($rowsNow.Count -gt 0 -and -not $rowsMs) { $rowsMs = $stopwatch.ElapsedMilliseconds - $openMs }
  if ($rowsNow.Count -eq 0 -and $nonEmpty.Count -gt 0 -and -not $contentMs) { $emptySeen = $true }
  if ($rowsNow.Count -ne $lastCount) { $lastCount = $rowsNow.Count; $stableSince = $stopwatch.ElapsedMilliseconds }
  $stableFor = if ($stableSince) { $stopwatch.ElapsedMilliseconds - $stableSince } else { 0 }
  if ($rowsNow.Count -gt 0 -and $stableFor -ge 700 -and -not $paintedMs) { $paintedMs = $stopwatch.ElapsedMilliseconds - $openMs; break }
  if ($rowsNow.Count -gt 0 -and $stableFor -ge (1000 * $SettleSeconds)) { $paintedMs = $stopwatch.ElapsedMilliseconds - $openMs; break }
  Start-Sleep -Milliseconds 50
}

Stop-Process -Id $app.Id -Force -ErrorAction SilentlyContinue
foreach ($p in (Get-Process | Where-Object { $_.Name -like 'gentle-desk*' -and $_.StartTime -gt (Get-Date).AddMinutes(-3) })) {
  Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
}

$result = [ordered]@{
  label = 'chat-open'
  pi_home = $PiHome
  session = $target.Current.Name
  open_ms = $openMs
  first_text_ms = $contentMs
  first_content_row_ms = $rowsMs
  painted_ms = $paintedMs
  empty_frame_before_content = $emptySeen
  rows_at_paint = $lastCount
  measured_at = (Get-Date).ToString('o')
}
$out = if ($OutJson) { $OutJson } else { '.verify/open.json' }
New-Item -ItemType Directory -Force -Path (Split-Path $out -Parent) | Out-Null
$result | ConvertTo-Json -Depth 4 | Set-Content -Encoding UTF8 $out
$result | ConvertTo-Json -Depth 4
