param([string]$Out)
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Drawing, System.Windows.Forms
Add-Type -Namespace W -Name U -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetForegroundWindow(System.IntPtr h); [DllImport("user32.dll")] public static extern System.IntPtr GetForegroundWindow(); [DllImport("user32.dll")] public static extern bool ShowWindow(System.IntPtr h,int c);'
$A=[System.Windows.Automation.AutomationElement]
$w=$null; for($i=0;$i -lt 60 -and -not $w;$i++){ foreach($x in $A::RootElement.FindAll('Children',[System.Windows.Automation.Condition]::TrueCondition)){ if($x.Current.Name -eq 'Gentle Shell Desktop'){$w=$x;break} }; if(-not $w){Start-Sleep -Milliseconds 100} }
if(-not $w){ 'no window'; exit 1 }
$h=[System.IntPtr]$w.Current.NativeWindowHandle
[W.U]::ShowWindow($h,9) | Out-Null; [W.U]::SetForegroundWindow($h) | Out-Null; Start-Sleep -Milliseconds 300
function Snap($name){
  if([W.U]::GetForegroundWindow() -ne $h){ "not foreground, no capture for $name"; return }
  $r=$w.Current.BoundingRectangle
  $bmp=New-Object System.Drawing.Bitmap([int]$r.Width,[int]$r.Height)
  $g=[System.Drawing.Graphics]::FromImage($bmp); $g.CopyFromScreen([int]$r.X,[int]$r.Y,0,0,$bmp.Size)
  $bmp.Save("$Out-$name.png"); "saved $name"
}
Snap 'a'; Start-Sleep -Milliseconds 700; Snap 'b'; Start-Sleep -Seconds 3; Snap 'c'
