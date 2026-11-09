# Counts UI Automation (UIA) elements reachable from one app's main window:
# total elements, those with a Name, and where thread/row text is exposed.
# Declared limits: control view only, capped at 12000 nodes and depth 30, so
# big trees are seen through a keyhole rather than measured exhaustively.
param(
  [Parameter(Mandatory = $true)][int]$AppPid,
  [int]$MaxNodes = 12000,
  [int]$MaxDepth = 30
)

Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

$root = [System.Windows.Automation.AutomationElement]::RootElement
$cond = New-Object System.Windows.Automation.PropertyCondition(
  [System.Windows.Automation.AutomationElement]::ProcessIdProperty, $AppPid)
$win = $root.FindFirst([System.Windows.Automation.TreeScope]::Children, $cond)
if (-not $win) { Write-Output '{"error":"no UIA window for this pid"}'; exit }

$walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
$stats = @{ total = 0; named = 0; textPattern = 0; gridOrList = 0; maxDepth = 0; timedOut = $false }
$names = New-Object System.Collections.Generic.List[string]
$deadline = (Get-Date).AddSeconds(30)

function Walk($el, $depth) {
  if ($depth -gt $MaxDepth -or (Get-Date) -gt $deadline) { return }
  $script:countTotal = $script:countTotal + 1
  if ($script:countTotal -gt $MaxNodes) { $stats.timedOut = $true; return }
  if ($depth -gt $stats.maxDepth) { $stats.maxDepth = $depth }
  try {
    $name = $el.Current.Name
    if ($name) { $stats.named += 1; if ($names.Count -lt 40) { $names.Add($name.ToString()) } }
    $textCapable = $false
    if ($el.TryGetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern, [ref]$textCapable)) {
      $stats.textPattern += 1
    }
    if ($el.Current.ControlType.ProgrammaticName -match 'List|Tree|DataItem|ListItem') {
      $stats.gridOrList += 1
    }
  } catch { }
  $child = $walker.GetFirstChild($el)
  while ($child) {
    Walk $child ($depth + 1)
    $child = $walker.GetNextSibling($child)
  }
}

$script:countTotal = 0
try {
  $dummy = $null
  $stats.textOnWindow = $win.TryGetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern, [ref]$dummy)
} catch { $stats.textOnWindow = $false }
try { $stats.windowName = $win.Current.Name } catch { $stats.windowName = '' }
Walk $win 0
$stats.total = $script:countTotal
$stats.namesSample = @($names | Select-Object -First 40)
$stats | ConvertTo-Json -Depth 3 -Compress
exit
