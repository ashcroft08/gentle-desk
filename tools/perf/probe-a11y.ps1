$ErrorActionPreference='Stop'
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Windows.Forms
$A=[System.Windows.Automation.AutomationElement]; $CT=[System.Windows.Automation.ControlType]
$R='C:\Users\josep\dev\gentle-desk-t9'; $S="$R\.verify\a11y\scratch"
Remove-Item -Recurse -Force $S -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$S\appdata\gentle-desk","$S\local","$S\home","$S\ws" | Out-Null
@{version=1;theme='system';workspaces=@(@{path="$S\ws";collapsed=$false})} | ConvertTo-Json -Depth 4 | Set-Content "$S\appdata\gentle-desk\config.json"
$env:GENTLE_SHELL_BIN="$R\tools\fake-gentle-shell\fake.mjs"; $env:APPDATA="$S\appdata"; $env:LOCALAPPDATA="$S\local"
Start-Process "$R\target\release\gentle-desk-app.exe" -ArgumentList '--home',"$S\home" | Out-Null
function Find($root,$type,$name){ $c=New-Object System.Windows.Automation.AndCondition((New-Object System.Windows.Automation.PropertyCondition($A::ControlTypeProperty,$type)),(New-Object System.Windows.Automation.PropertyCondition($A::NameProperty,$name))); $root.FindFirst('Descendants',$c) }
$w=$null; for($i=0;$i -lt 40 -and -not $w;$i++){ $w=$A::RootElement.FindFirst('Children',(New-Object System.Windows.Automation.PropertyCondition($A::NameProperty,'gentle-desk'))); if(-not $w){Start-Sleep -Milliseconds 500} }
$appPid=$w.Current.ProcessId
function Pids($p){ $all=Get-CimInstance Win32_Process; $f=@($p); $q=@($p); while($q.Count){ $c=$q[0]; $q=$q[1..($q.Count)]; foreach($k in ($all|?{$_.ParentProcessId -eq $c})){ if($f -notcontains [int]$k.ProcessId){$f+=[int]$k.ProcessId;$q+=[int]$k.ProcessId} } }; $f }
function Cpu(){ $t=0.0; foreach($i in (Pids $appPid)){ $p=Get-Process -Id $i -EA SilentlyContinue; if($p){$t+=$p.TotalProcessorTime.TotalSeconds} }; $t }
$b=$null; for($i=0;$i -lt 40 -and -not $b;$i++){ $b=Find $w $CT::Button 'New chat in ws'; if(-not $b){Start-Sleep -Milliseconds 500} }
$b.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke(); Start-Sleep 4
function Send($text){
  $ed=$null; for($i=0;$i -lt 20 -and -not $ed;$i++){ $ed=Find $w $CT::Edit 'Message'; if(-not $ed){Start-Sleep -Milliseconds 300} }
  $ed.SetFocus(); Start-Sleep -Milliseconds 400
  [System.Windows.Forms.SendKeys]::SendWait($text); Start-Sleep -Milliseconds 400
  $s=Find $w $CT::Button 'Send'; $s.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
}
function WaitIdle($max){ $sw=[Diagnostics.Stopwatch]::StartNew(); while($sw.Elapsed.TotalSeconds -lt $max){ Start-Sleep -Milliseconds 700; if(Find $w $CT::Button 'Stop'){continue}; $s=Find $w $CT::Button 'Send'; if($s){return $true} }; $false }

Send 'echo hola'; Start-Sleep 3
$out=[ordered]@{}
$unnamed=@(); $named=0
foreach($t in @($CT::Button,$CT::Edit,$CT::TreeItem,$CT::TabItem,$CT::Hyperlink,$CT::Tree,$CT::Document)){
  foreach($e in $w.FindAll('Descendants',(New-Object System.Windows.Automation.PropertyCondition($A::ControlTypeProperty,$t)))){
    if([string]::IsNullOrWhiteSpace($e.Current.Name)){ $unnamed+=($t.ProgrammaticName+' id='+$e.Current.AutomationId) } else { $named++ } } }
$out.interactive_named=$named; $out.interactive_unnamed=$unnamed
try{ (Find $w $CT::Document "gentle-desk").SetFocus() }catch{ "doc focus failed" }; Start-Sleep -Milliseconds 500
$seq=@(); $first=$null
for($i=0;$i -lt 40;$i++){
  [System.Windows.Forms.SendKeys]::SendWait('{TAB}'); Start-Sleep -Milliseconds 250
  $f=$A::FocusedElement; $label=$f.Current.ControlType.ProgrammaticName.Replace('ControlType.','')+': '+$f.Current.Name
  if($first -and $label -eq $first){ $seq+="(back to) $label"; break }
  if(-not $first){$first=$label}; $seq+=$label }
$out.tab_order=$seq
foreach($i in (Pids $appPid)){ Stop-Process -Id $i -Force -EA SilentlyContinue }
$out | ConvertTo-Json -Depth 4
