$ErrorActionPreference='Stop'
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Windows.Forms
$A=[System.Windows.Automation.AutomationElement]; $CT=[System.Windows.Automation.ControlType]
$R='C:\Users\josep\dev\gentle-desk-t9'; $S="$R\.verify\stream\scratch"
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
$out=[ordered]@{}
foreach($phase in 'small','after-flood'){
  if($phase -eq 'after-flood'){ Send 'flood'; Start-Sleep 2; $out.flood_idle=WaitIdle 120 }
  $c0=Cpu; $t0=Get-Date
  Send 'stream 20'; Start-Sleep -Milliseconds 1500
  $mid0=Cpu; Start-Sleep 15; $mid1=Cpu
  $idle=WaitIdle 60; $c1=Cpu; $secs=((Get-Date)-$t0).TotalSeconds
  $out[$phase]=[ordered]@{ total_cpu_s=[math]::Round($c1-$c0,2); wall_s=[math]::Round($secs,1); steady_15s_cpu_s=[math]::Round($mid1-$mid0,2); finished=$idle }
}
$out.rows=@($w.FindAll('Descendants',(New-Object System.Windows.Automation.PropertyCondition($A::ControlTypeProperty,$CT::Group)))).Count
foreach($i in (Pids $appPid)){ Stop-Process -Id $i -Force -EA SilentlyContinue }
$out | ConvertTo-Json -Depth 4
