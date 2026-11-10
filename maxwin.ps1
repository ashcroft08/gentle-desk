Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$A=[System.Windows.Automation.AutomationElement]
$w=$null; for($i=0;$i -lt 40 -and -not $w;$i++){ foreach($x in $A::RootElement.FindAll('Children',[System.Windows.Automation.Condition]::TrueCondition)){ if($x.Current.Name -eq 'Gentle Shell Desktop'){$w=$x;break} }; if(-not $w){Start-Sleep -Milliseconds 200} }
$w.GetCurrentPattern([System.Windows.Automation.WindowPattern]::Pattern).SetWindowVisualState('Maximized'); "maximized " + $w.Current.BoundingRectangle
