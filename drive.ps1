param([string]$Step)
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$A=[System.Windows.Automation.AutomationElement]
function Win { $r=$A::RootElement; foreach($w in $r.FindAll('Children',[System.Windows.Automation.Condition]::TrueCondition)){ if($w.Current.Name -like '*entle*desk*'){return $w} } }
function Dump($el,$d,$max){ if($d -gt $max){return}; $c=$el.Current; if($c.Name -or $c.ControlType.ProgrammaticName -match 'Button|Edit|ListItem|TreeItem'){ ('  '*$d)+$c.ControlType.ProgrammaticName.Replace('ControlType.','')+' | '+$c.Name+' | '+$c.AutomationId }; foreach($k in $el.FindAll('Children',[System.Windows.Automation.Condition]::TrueCondition)){ Dump $k ($d+1) $max } }
$w=$null; for($i=0;$i -lt 20 -and -not $w;$i++){ $w=Win; if(-not $w){Start-Sleep 1} }
if(-not $w){ 'no window'; foreach($x in $A::RootElement.FindAll('Children',[System.Windows.Automation.Condition]::TrueCondition)){$x.Current.Name}; exit }
if($Step -eq 'dump'){ Dump $w 0 14 }
function Find($root,$type,$name){ $c=New-Object System.Windows.Automation.AndCondition((New-Object System.Windows.Automation.PropertyCondition($A::ControlTypeProperty,$type)),(New-Object System.Windows.Automation.PropertyCondition($A::NameProperty,$name))); $root.FindFirst('Descendants',$c) }
if($Step -eq 'chat'){
  $b=Find $w ([System.Windows.Automation.ControlType]::Button) 'New chat in ws2'
  $b.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke(); Start-Sleep 6
  $ed=$null; foreach($e in $w.FindAll('Descendants',(New-Object System.Windows.Automation.PropertyCondition($A::ControlTypeProperty,[System.Windows.Automation.ControlType]::Edit)))){ $ed=$e }
  'edit: '+$ed.Current.Name
  $ed.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).SetValue('Responde solo OK'); Start-Sleep 1
  $s=Find $w ([System.Windows.Automation.ControlType]::Button) 'Send'
  'send: '+($s -ne $null)+' enabled '+$s.Current.IsEnabled
  $s.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
}
if($Step -eq 'type'){
  Add-Type -AssemblyName System.Windows.Forms
  $ed=$null; foreach($e in $w.FindAll('Descendants',(New-Object System.Windows.Automation.PropertyCondition($A::ControlTypeProperty,[System.Windows.Automation.ControlType]::Edit)))){ $ed=$e }
  $ed.SetFocus(); Start-Sleep 1
  [System.Windows.Forms.SendKeys]::SendWait('stream 20'); Start-Sleep 1
  $s=Find $w ([System.Windows.Automation.ControlType]::Button) 'Send'
  'send enabled '+$s.Current.IsEnabled
  if($s.Current.IsEnabled){ $s.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke() }
}
if($Step -eq 'scroll'){
  $t=Find $w ([System.Windows.Automation.ControlType]::Tree) 'Workspaces'
  $p=$t; $sp=$null
  for($i=0;$i -lt 6 -and $p;$i++){ try{ $sp=$p.GetCurrentPattern([System.Windows.Automation.ScrollPattern]::Pattern); break }catch{ $p=[System.Windows.Automation.TreeWalker]::ControlViewWalker.GetParent($p) } }
  if($sp){ $sp.SetScrollPercent(-1,$(if($env:TOP){0}else{100})); Start-Sleep 2; 'scrolled' } else { 'no scroll pattern' }
  Dump $t 0 4
}
if($Step -eq 'shot'){
  Add-Type -AssemblyName System.Drawing, System.Windows.Forms
  $r=$w.Current.BoundingRectangle
  $bmp=New-Object System.Drawing.Bitmap([int]$r.Width,[int]$r.Height)
  $g=[System.Drawing.Graphics]::FromImage($bmp); $g.CopyFromScreen([int]$r.X,[int]$r.Y,0,0,$bmp.Size)
  $bmp.Save('C:\Users\josep\dev\gentle-desk\.verify\real\shot.png'); 'saved '+$r
}
