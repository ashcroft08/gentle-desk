param([string]$Name)
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$A=[System.Windows.Automation.AutomationElement]
$w=$null; for($i=0;$i -lt 40 -and -not $w;$i++){ foreach($x in $A::RootElement.FindAll('Children',[System.Windows.Automation.Condition]::TrueCondition)){ if($x.Current.Name -eq 'Gentle Shell Desktop'){$w=$x;break} }; if(-not $w){Start-Sleep -Milliseconds 200} }
$c=New-Object System.Windows.Automation.AndCondition((New-Object System.Windows.Automation.PropertyCondition($A::ControlTypeProperty,[System.Windows.Automation.ControlType]::Button)),(New-Object System.Windows.Automation.PropertyCondition($A::NameProperty,$Name)))
$b=$null; for($i=0;$i -lt 30 -and -not $b;$i++){ $b=$w.FindFirst('Descendants',$c); if(-not $b){Start-Sleep -Milliseconds 200} }
if(-not $b){ "button '$Name' not found"; exit 1 }
try { $b.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke(); "invoked $Name" }
catch { try { $b.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Toggle(); "toggled $Name" } catch { "COULD NOT PRESS $Name"; exit 1 } }
