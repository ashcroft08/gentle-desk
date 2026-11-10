param([string]$Like='Pregunta')
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$A=[System.Windows.Automation.AutomationElement]; $CT=[System.Windows.Automation.ControlType]
$w=$null; foreach($x in $A::RootElement.FindAll('Children',[System.Windows.Automation.Condition]::TrueCondition)){ if($x.Current.Name -eq 'Gentle Shell Desktop'){$w=$x;break} }
$hit=$null
foreach($e in $w.FindAll('Descendants',(New-Object System.Windows.Automation.PropertyCondition($A::ControlTypeProperty,$CT::TreeItem)))){ if($e.Current.Name -like "*$Like*"){ $hit=$e; break } }
if(-not $hit){ 'row not found'; exit 1 }
try { $hit.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern).Select(); "selected "+$hit.Current.Name }
catch { try { $hit.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke(); "invoked "+$hit.Current.Name } catch { 'could not open: '+$hit.Current.Name } }
