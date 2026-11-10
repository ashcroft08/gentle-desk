# Draws the two images the NSIS installer shows from the app icon:
#   sidebar.bmp  164x314  welcome and finish pages: the rose on the app's dark colour
#   header.bmp   150x57   the header of the other pages (NSIS draws it on white)
# Output is 24-bit BMP, which is what NSIS expects.
#
#   powershell -File tools/installer/make-images.ps1
param(
  [string]$Icon = (Join-Path $PSScriptRoot '..\..\apps\desktop\app-icon.png'),
  [string]$OutDir = (Join-Path $PSScriptRoot '..\..\apps\desktop\src-tauri\installer')
)
Add-Type -AssemblyName System.Drawing
$rose = [System.Drawing.Image]::FromFile((Resolve-Path $Icon))
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

function Draw([int]$w, [int]$h, [System.Drawing.Color]$back, [int]$box, [int]$y, [string]$name) {
  $bmp = New-Object System.Drawing.Bitmap($w, $h, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.Clear($back)
  $g.InterpolationMode = 'HighQualityBicubic'
  $g.SmoothingMode = 'HighQuality'
  # Fit the square icon into a box x box area, centred horizontally.
  $x = [int](($w - $box) / 2)
  $g.DrawImage($rose, $x, $y, $box, $box)
  $bmp.Save((Join-Path (Resolve-Path $OutDir) $name), [System.Drawing.Imaging.ImageFormat]::Bmp)
  $g.Dispose(); $bmp.Dispose()
}

Draw 164 314 ([System.Drawing.Color]::FromArgb(14, 17, 22)) 150 82 'sidebar.bmp'
Draw 150 57 ([System.Drawing.Color]::White) 53 2 'header.bmp'
$rose.Dispose()
"wrote sidebar.bmp and header.bmp to $OutDir"
