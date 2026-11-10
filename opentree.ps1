Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$A=[System.Windows.Automation.AutomationElement]; $CT=[System.Windows.Automation.ControlType]
$w=$null; foreach($x in $A::RootElement.FindAll('Children',[System.Windows.Automation.Condition]::TrueCondition)){ if($x.Current.Name -eq 'Gentle Shell Desktop'){$w=$x;break} }
$item=$w.FindFirst('Descendants',(New-Object System.Windows.Automation.PropertyCondition($A::ControlTypeProperty,$CT::TreeItem)))
if(-not $item){'no tree item'; exit}
try { $item.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern).Select(); 'selected' } catch { try { $item.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke(); 'invoked' } catch { 'could not open' } }
