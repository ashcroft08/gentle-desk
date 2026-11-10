Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$A=[System.Windows.Automation.AutomationElement]
$w=$null; foreach($x in $A::RootElement.FindAll('Children',[System.Windows.Automation.Condition]::TrueCondition)){ if($x.Current.Name -eq 'Gentle Shell Desktop'){$w=$x;break} }
if(-not $w){'no window';exit}
foreach($e in $w.FindAll('Descendants',(New-Object System.Windows.Automation.PropertyCondition($A::ControlTypeProperty,[System.Windows.Automation.ControlType]::Button)))){ 'Button | '+$e.Current.Name+' | enabled='+$e.Current.IsEnabled }
foreach($e in $w.FindAll('Descendants',(New-Object System.Windows.Automation.PropertyCondition($A::ControlTypeProperty,[System.Windows.Automation.ControlType]::Header)))){ 'Header | '+$e.Current.Name }
foreach($e in $w.FindAll('Descendants',(New-Object System.Windows.Automation.PropertyCondition($A::ControlTypeProperty,[System.Windows.Automation.ControlType]::TreeItem)))){ 'TreeItem | '+$e.Current.Name }
