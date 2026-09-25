Add-Type -AssemblyName System.Drawing
$ErrorActionPreference = 'Stop'

$W = 1680; $H = 1742
$bmp = New-Object System.Drawing.Bitmap($W, $H)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = 'AntiAlias'
$g.TextRenderingHint = 'AntiAliasGridFit'
$g.InterpolationMode = 'HighQualityBicubic'

function C([int]$a,[int]$r,[int]$g2,[int]$b) { return [System.Drawing.Color]::FromArgb($a,$r,$g2,$b) }
function HexC([string]$hex) {
  $h = $hex.TrimStart('#')
  return [System.Drawing.Color]::FromArgb(255, [Convert]::ToInt32($h.Substring(0,2),16), [Convert]::ToInt32($h.Substring(2,2),16), [Convert]::ToInt32($h.Substring(4,2),16))
}
function Brush($c) { return New-Object System.Drawing.SolidBrush($c) }
function RRect([single]$x,[single]$y,[single]$w,[single]$h,[single]$r) {
  if ($r -le 0.5) { $pp = New-Object System.Drawing.Drawing2D.GraphicsPath; $pp.AddRectangle((New-Object System.Drawing.RectangleF($x,$y,$w,$h))); return $pp }
  $p = New-Object System.Drawing.Drawing2D.GraphicsPath
  $d = [single]($r * 2)
  if ($d -gt $w) { $d = $w }; if ($d -gt $h) { $d = $h }
  $p.AddArc($x, $y, $d, $d, 180, 90)
  $p.AddArc($x + $w - $d, $y, $d, $d, 270, 90)
  $p.AddArc($x + $w - $d, $y + $h - $d, $d, $d, 0, 90)
  $p.AddArc($x, $y + $h - $d, $d, $d, 90, 90)
  $p.CloseFigure()
  return $p
}
function FillR($x,$y,$w,$h,$r,$color) { $p = RRect $x $y $w $h $r; $b = Brush $color; $g.FillPath($b, $p); $b.Dispose(); $p.Dispose() }
function StrokeR($x,$y,$w,$h,$r,$color,[single]$width) { $p = RRect $x $y $w $h $r; $pen = New-Object System.Drawing.Pen($color, $width); $g.DrawPath($pen, $p); $pen.Dispose(); $p.Dispose() }
function Shadow($x,$y,$w,$h,$r) { for ($i = 12; $i -ge 1; $i--) { FillR ($x-$i) ($y+$i*0.55) ($w+(2*$i)) ($h+(2*$i)) ($r+$i) (C 5 0 0 0) } }
function Txt($text,$x,$y,$w,$h,$size,$color,[string]$family,[string]$style,[string]$align) {
  $f = New-Object System.Drawing.Font($family, $size, ([System.Drawing.FontStyle]$style), [System.Drawing.GraphicsUnit]::Pixel)
  $sf = New-Object System.Drawing.StringFormat
  $sf.Alignment = ([System.Drawing.StringAlignment]$align)
  $sf.LineAlignment = 'Center'
  $sf.FormatFlags = 'NoWrap'
  $b = Brush $color
  $g.DrawString($text, $f, $b, (New-Object System.Drawing.RectangleF($x,$y,$w,$h)), $sf)
  $b.Dispose(); $sf.Dispose(); $f.Dispose()
}
function Icon($glyph,$x,$y,$w,$h,$size,$color,[string]$align) { Txt $glyph $x $y $w $h $size $color 'Segoe Fluent Icons' 'Regular' $align }
function TW($text,$size,[string]$style) {
  $f = New-Object System.Drawing.Font($YH, $size, ([System.Drawing.FontStyle]$style), [System.Drawing.GraphicsUnit]::Pixel)
  $w = $g.MeasureString($text,$f).Width
  $f.Dispose()
  return $w
}
function A([System.Drawing.Color]$c,[int]$alpha) { return [System.Drawing.Color]::FromArgb($alpha, $c.R, $c.G, $c.B) }

$YH = 'Microsoft YaHei UI'
$work = HexC '#e53e3e'; $shortC = HexC '#38a169'; $longC = HexC '#3182ce'; $pauseC = HexC '#718096'
$surfaceL = HexC '#ffffff'; $softL = HexC '#f0f0f3'; $borderL = HexC '#e3e3e8'; $textL = HexC '#1d1d1f'; $mutedL = HexC '#8b8b93'
$surfaceD = HexC '#232b3b'; $softD = HexC '#2d3648'; $borderD = HexC '#3b4557'; $textD = HexC '#f7fafc'; $mutedD = HexC '#9aa6b8'

function Panel($x,$y,$w,$h,$title,$sub) {
  FillR $x $y $w $h 14 (C 18 255 255 255)
  StrokeR $x $y $w $h 14 (C 26 255 255 255) 1
  Txt $title ($x+22) ($y+10) ($w-44) 24 18 (C 235 235 242 250) $YH 'Bold' 'Near'
  if ($sub) { Txt $sub ($x+22) ($y+34) ($w-44) 20 13 (C 150 190 205 225) $YH 'Regular' 'Near' }
}
function Wallpaper($x,$y,$w,$h,[bool]$dark) {
  $rect = New-Object System.Drawing.RectangleF($x,$y,$w,$h)
  $g.SetClip($rect)
  $c1 = (C 255 236 240 246); $c2 = (C 255 198 210 224)
  if ($dark) { $c1 = (C 255 38 50 66); $c2 = (C 255 11 14 19) }
  $lg = New-Object System.Drawing.Drawing2D.LinearGradientBrush($rect, $c1, $c2, 35)
  $g.FillRectangle($lg, $x, $y, $w, $h); $lg.Dispose()
  $path = New-Object System.Drawing.Drawing2D.GraphicsPath
  $path.AddEllipse(($x - $w*0.15), ($y - $h*0.4), ($w*0.95), ($h*1.15))
  $pb = New-Object System.Drawing.Drawing2D.PathGradientBrush($path)
  if ($dark) { $pb.CenterColor = (C 44 62 152 162); $pb.SurroundColors = @((C 0 62 152 162)) } else { $pb.CenterColor = (C 38 124 172 236); $pb.SurroundColors = @((C 0 124 172 236)) }
  $g.FillPath($pb, $path); $pb.Dispose(); $path.Dispose()
  $g.ResetClip()
}
function GlowTxt($text,$x,$y,$w,$h,$size,$color,[System.Drawing.Color]$halo,[int]$ha) {
  $offs = @(@(-1.6,0),@(1.6,0),@(0,-1.6),@(0,1.6),@(-1.1,-1.1),@(1.1,-1.1),@(-1.1,1.1),@(1.1,1.1))
  foreach ($o in $offs) { Txt $text ($x+$o[0]) ($y+$o[1]) $w $h $size (A $halo $ha) $YH 'Bold' 'Center' }
  Txt $text $x $y $w $h $size $color $YH 'Bold' 'Center'
}
function Sub($text,$x,$y,$w,$h,$size,$color) { Txt $text $x $y $w $h $size $color $YH 'Regular' 'Center' }
function Note($text,$x,$y,$w,[int]$size,[System.Drawing.Color]$color) { Txt $text $x $y $w 22 $size $color $YH 'Regular' 'Near' }
function Pill($x,$y,$w,$h,$r) { FillR $x $y $w $h $r (A $surfaceL 236); StrokeR $x $y $w $h $r (A (HexC '#c9ccd4') 200) 1 }
function IconBtn($x,$y,$size,$glyph) {
  FillR $x $y $size $size 9 (A $surfaceL 236)
  StrokeR $x $y $size $size 9 (A (HexC '#c9ccd4') 200) 1
  Icon $glyph $x $y $size $size ($size*0.42) $mutedL 'Center'
}
function Btn($x,$y,$w,$h,$label,$glyph,$filled,[System.Drawing.Color]$color,[bool]$enabled) {
  if ($filled) { FillR $x $y $w $h 9 $color } else { Pill $x $y $w $h 9 }
  $fg = (C 255 255 255 255)
  if (-not $filled) { $fg = $textL; if (-not $enabled) { $fg = (A $mutedL 120) } }
  $iw = 14; $gp = 5
  $st = 'Regular'; if ($filled) { $st = 'Bold' }
  $lw = TW $label ($h*0.36) $st
  $sx = $x + ($w - ($iw + $gp + $lw))/2
  Icon $glyph $sx ($y+1) $iw $h ($h*0.33) $fg 'Center'
  Txt $label ($sx+$iw+$gp) $y ($lw+8) $h ($h*0.36) $fg $YH $st 'Near'
}

# 胶囊形态（= 参考图 1）：圆角面 + 活动行（上） + 数字（下） [+ 按钮行]
function Capsule([single]$x,[single]$y,[single]$s,[System.Drawing.Color]$stage,[string]$activity,[string]$time,[string]$module,[bool]$paused,[bool]$running,[bool]$showButtons,[bool]$dark) {
  $fw = 280*$s; $pad = 12*$s; $headH = 30*$s; $timeH = 56*$s; $btnH = 32*$s; $gap = 8*$s; $rad = 18*$s
  $fh = $pad + $headH + $gap + $timeH + $pad
  if ($showButtons) { $fh = $fh + $gap + $btnH }
  $surface = $surfaceL; $soft = $softL; $border = $borderL; $text = $textL; $muted = $mutedL; $alpha = 246
  if ($dark) { $surface = $surfaceD; $soft = $softD; $border = $borderD; $text = $textD; $muted = $mutedD; $alpha = 236 }
  Shadow $x $y $fw $fh $rad
  FillR $x $y $fw $fh $rad (A $surface $alpha)
  StrokeR $x $y $fw $fh $rad (A $border 210) 1

  $hy = $y + $pad
  $cy = $hy + $headH/2
  $dot = 8*$s
  $b = Brush $stage
  $g.FillEllipse($b, ($x+$pad), ($cy-$dot/2), $dot, $dot)
  $b.Dispose()
  $chip = 28*$s
  $openX = $x + $fw - $pad - $chip
  $editX = $openX - (6*$s) - $chip
  $nameX = $x + $pad + $dot + (7*$s)
  $nameW = $editX - (10*$s) - $nameX
  Txt $activity $nameX $hy $nameW $headH (13*$s) $text $YH 'Bold' 'Near'
  $nw = [Math]::Min((TW $activity (13*$s) 'Bold'), ($nameW - (20*$s)))
  Icon ([char]0xE70D) ($nameX + $nw + (2*$s)) $hy (14*$s) $headH (9*$s) $muted 'Center'
  FillR $editX $hy $chip $headH (8*$s) (A $soft 235)
  Icon ([char]0xE70F) $editX $hy $chip $headH (12*$s) $muted 'Center'
  FillR $openX $hy $chip $headH (8*$s) (A $soft 235)
  Icon ([char]0xE8A7) $openX $hy $chip $headH (12*$s) $muted 'Center'

  $ty = $hy + $headH + $gap
  $tc = $stage; if ($paused) { $tc = $pauseC }
  Txt $time $x $ty $fw $timeH (38*$s) $tc $YH 'Bold' 'Center'

  if (-not $showButtons) { return $fh }

  $by = $ty + $timeH + $gap
  $innerW = $fw - (2*$pad)
  $defs = @()
  if ($module -eq 'pomodoro') { $defs = @(@('E768','开始'),@('E71A','结束'),@('E72C','重置')) } else { $defs = @(@('E768','开始'),@('E71A','结束')) }
  $n = $defs.Count
  $bw = ($innerW - ($n-1)*(8*$s)) / $n
  for ($i = 0; $i -lt $n; $i++) {
    $bx = $x + $pad + $i*($bw + (8*$s))
    $label = $defs[$i][1]
    $gl = $defs[$i][0]
    $filled = ($i -eq 0)
    $enabled = $true
    if ($i -eq 0) {
      if ($paused) { $label = '继续' }
      if ($running) { $label = '暂停'; $gl = 'E769' }
    } elseif ($module -eq 'pomodoro' -and $i -eq 2 -and -not $running) { $enabled = $false }
    if ($filled) {
      FillR $bx $by $bw $btnH (10*$s) $stage
      $fg = (C 255 255 255 255)
    } else {
      FillR $bx $by $bw $btnH (10*$s) (A $soft 225)
      $fg = $text; if (-not $enabled) { $fg = (A $muted 120) }
    }
    $iw = 15*$s; $gp = 5*$s
    $st = 'Regular'; if ($filled) { $st = 'Bold' }
    $lw = TW $label (12*$s) $st
    $sx = $bx + ($bw - ($iw + $gp + $lw))/2
    Icon ([char][Convert]::ToInt32($gl,16)) $sx ($by+(1*$s)) $iw $btnH (11*$s) $fg 'Center'
    Txt $label ($sx+$iw+$gp) $by ($lw+8) $btnH (12*$s) $fg $YH $st 'Near'
  }
  return $fh
}

# ---------------- 背景 ----------------
$bgRect = New-Object System.Drawing.Rectangle(0,0,$W,$H)
$lin = New-Object System.Drawing.Drawing2D.LinearGradientBrush($bgRect, (C 255 14 18 24), (C 255 30 42 56), 60)
$g.FillRectangle($lin, $bgRect); $lin.Dispose()
$glow = New-Object System.Drawing.Drawing2D.GraphicsPath
$glow.AddEllipse(-300, -460, 1200, 950)
$pg = New-Object System.Drawing.Drawing2D.PathGradientBrush($glow)
$pg.CenterColor = (C 22 80 130 180); $pg.SurroundColors = @((C 0 80 130 180))
$g.FillPath($pg, $glow); $pg.Dispose(); $glow.Dispose()

Txt '番茄钟 · 无边框 / 胶囊 悬浮形态 设计参考' 80 40 1400 44 34 (C 255 240 244 250) $YH 'Bold' 'Near'
Txt '无皮肤 · 无拟物番茄 · 无图片资源 · 透明背景 · 无边框 · 置顶 · 不进任务栏 · 颜色与字号全部跟随设置面板' 80 84 1560 26 17 (C 200 190 205 225) $YH 'Regular' 'Near'

# ---------------- A 无边框待机 ----------------
Panel 80 150 760 470 'A · 无边框形态 · 待机' '只画数字：透明底、无卡片、无边框、无活动名、无按钮'
Wallpaper 104 216 356 300 $false
Wallpaper 460 216 356 300 $true
GlowTxt '24:12' 104 216 356 300 64 $work (C 255 0 0 0) 55
GlowTxt '24:12' 460 216 356 300 64 $work (C 255 0 0 0) 70
Txt '浅色壁纸' 116 220 120 24 13 (C 220 130 140 155) $YH 'Regular' 'Near'
Txt '深色壁纸' 472 220 120 24 13 (C 190 225 235 245) $YH 'Regular' 'Near'
Note '待机态只画数字或图形（颜色 = 阶段色），活动名与所有操作按钮全部隐藏' 104 526 712 14 (C 200 215 230 245)
Note '可拖拽区 = 内容外框 + 8 DIP 内边距；其余透明像素返回 HTTRANSPARENT，不挡桌面上的其他操作' 104 550 712 13 (C 150 175 195 220)
Note '零皮肤 / 零图片：数字与图标全部矢量绘制，图标取系统自带的 Segoe Fluent Icons' 104 574 712 13 (C 150 175 195 220)

# ---------------- B 无边框展开 ----------------
Panel 880 150 720 470 'B · 无边框形态 · 单击展开' '单击 = 展开 / 收起 · 拖拽 = 移动（8px 阈值区分）'
Wallpaper 904 216 672 300 $true
GlowTxt '24:12' 904 222 672 84 54 $work (C 255 0 0 0) 70
$nameTxt = '写代码'
$dW = 8; $chipS = 34; $chipGap = 6
$nW = TW $nameTxt 14 'Bold'
$rowW = $dW + 6 + $nW + 4 + 14 + 34 + ($chipS*2 + $chipGap)
$rowX = 1240 - ($rowW/2)
$ay = 316
$b = Brush $work
$g.FillEllipse($b, $rowX, ($ay + (30-$dW)/2), $dW, $dW); $b.Dispose()
Txt $nameTxt ($rowX + $dW + 6) $ay ($nW+8) 30 14 (C 255 255 255 255) $YH 'Bold' 'Near'
Icon ([char]0xE70D) ($rowX + $dW + 6 + $nW + 4) $ay 14 30 9 (C 190 215 230 235) 'Center'
$chX = 1240 + ($rowW/2) - ($chipS*2 + $chipGap)
IconBtn $chX $ay $chipS ([char]0xE70F)
IconBtn ($chX + $chipS + $chipGap) $ay $chipS ([char]0xE8A7)
Btn 1062 362 112 32 '开始' ([char]0xE768) $true $work $true
Btn 1184 362 112 32 '结束' ([char]0xE71A) $false $work $true
Btn 1306 362 112 32 '重置' ([char]0xE72C) $false $work $false
Btn 1123 406 112 28 '开始' ([char]0xE768) $true $work $true
Btn 1245 406 112 28 '结束' ([char]0xE71A) $false $work $true
Txt '上排 = 番茄钟 3 键；这排 = 计时模式，同一位置换成 2 键' 1062 440 356 22 12 (C 205 220 235 245) $YH 'Regular' 'Center'
Txt '展开区 3 秒无操作自动收起（可选）' 1062 470 356 22 12 (C 170 195 215 225) $YH 'Regular' 'Center'
Note '展开时数字下方浮出「活动行 + 按钮行」，不包裹数字 → 仍是「无边框」观感' 904 526 672 13 (C 200 215 230 245)
Note '按钮动作与主窗口同源（开始/暂停/继续 · 结束 · 重置），无新增业务逻辑' 904 550 672 13 (C 150 175 195 220)
Note '本体单击只用于展开/收起，不触发开始/暂停' 904 574 672 13 (C 150 175 195 220)

# ---------------- C 胶囊待机 ----------------
Panel 80 650 760 500 'C · 胶囊形态 · 待机（= 参考图 1）' '圆角半透明面；活动行在上、数字在下；开始/结束/重置全部隐藏'
Wallpaper 104 716 712 400 $true
Capsule 180 798 2.0 $work '写代码' '24:12' 'pomodoro' $false $true $false $false | Out-Null
Txt '待机态：只有活动与时间，操作按钮一条都不出现' 104 1058 712 22 13 (C 200 215 230 245) $YH 'Regular' 'Center'

# ---------------- D 胶囊点击后 ----------------
Panel 880 650 720 500 'D · 胶囊形态 · 单击后（浮出按钮行）' '按钮随模块切换：番茄钟 3 键 / 计时 2 键'
Wallpaper 904 716 672 330 $true
Capsule 910 782 1.25 $work '写代码' '24:12' 'pomodoro' $false $true $true $false | Out-Null
Capsule 1290 802 1.0 $work '阅读' '12:30' 'timer' $false $false $true $true | Out-Null
Txt '左：番茄钟（浅色主题）开始/暂停 · 结束 · 重置　　右：计时（深色主题）开始/暂停 · 结束' 904 1056 672 22 12 (C 215 230 245 250) $YH 'Regular' 'Center'
Note '收起后，按钮行所占区域在窗口里是透明像素 → 不挡桌面，也不需要重分配窗口' 904 1084 672 12 (C 170 195 215 225)
Note '活动行：点名字 ▾ 切换预设 / 直接输入自定义；✎ 改名；⤢ 切到番茄钟视图并把主窗口前置' 904 1108 672 12 (C 170 195 215 225)

# ---------------- E 图形模式 ----------------
Panel 80 1180 1000 380 'E · 图形模式（跟随设置面板）' '简洁 / 条形 / 环形 / 隐藏文字 —— 无边框形态下直接浮在壁纸上'
Wallpaper 104 1246 952 240 $true
$e1 = 223; $e2 = 461; $e3 = 699; $e4 = 937; $cyE = 1361
GlowTxt '24:12' ($e1-110) 1246 220 240 34 $work (C 255 0 0 0) 70
GlowTxt '24:12' ($e2-110) 1256 220 56 26 $work (C 255 0 0 0) 70
FillR ($e2-80) 1356 160 10 5 (A (HexC '#ffffff') 60)
FillR ($e2-80) 1356 99 10 5 $work
foreach ($c in @($e3, $e4)) {
  $pen0 = New-Object System.Drawing.Pen((A $work 60), 9)
  $g.DrawArc($pen0, ($c-58), ($cyE-58), 116, 116, -90, 359.9); $pen0.Dispose()
  $pen1 = New-Object System.Drawing.Pen($work, 9)
  $pen1.StartCap = 'Round'; $pen1.EndCap = 'Round'
  $g.DrawArc($pen1, ($c-58), ($cyE-58), 116, 116, -90, 223); $pen1.Dispose()
}
GlowTxt '24:12' ($e3-110) 1331 220 60 22 $work (C 255 0 0 0) 70
Sub '简洁（只数字）' ($e1-119) 1496 238 24 13 (C 205 220 235 245)
Sub '条形（数字 + 进度）' ($e2-119) 1496 238 24 13 (C 205 220 235 245)
Sub '环形（外圈进度 + 数字）' ($e3-119) 1496 238 24 13 (C 205 220 235 245)
Sub '隐藏文字（只留图形）' ($e4-119) 1496 238 24 13 (C 205 220 235 245)
Note '环形 / 条形进度 = 剩余比例，从 12 点方向顺时针（与主窗口番茄钟视图一致）；「隐藏界面文字」开启时数字不绘制，只留图形' 104 1526 952 13 (C 150 175 195 220)

# ---------------- F 阶段色 ----------------
Panel 1100 1180 580 380 'F · 阶段色与暂停' '数字颜色 = 主题的 work / short / long / pause'
Wallpaper 1124 1240 532 246 $true
$f1 = 1257; $f2 = 1523
GlowTxt '24:12' ($f1-133) 1254 266 70 30 $work (C 255 0 0 0) 70
Sub '专注' ($f1-133) 1324 266 22 11 (C 225 238 246 255)
GlowTxt '04:38' ($f2-133) 1254 266 70 30 $shortC (C 255 0 0 0) 70
Sub '短休息' ($f2-133) 1324 266 22 11 (C 225 238 246 255)
GlowTxt '14:02' ($f1-133) 1366 266 70 30 $longC (C 255 0 0 0) 70
Sub '长休息' ($f1-133) 1436 266 22 11 (C 225 238 246 255)
GlowTxt '24:12' ($f2-133) 1366 266 70 30 $pauseC (C 255 0 0 0) 70
Sub '已暂停（pause 色）' ($f2-133) 1436 266 22 11 (C 225 238 246 255)
Note '暂停时数字转 pause 色；切换主题 / 暗色 / 自定义配色后小图标立刻同步重绘' 1124 1496 532 12 (C 150 175 195 220)

# ---------------- 说明 ----------------
Note '形态二选一：无边框（不绘制任何底色，只画数字/图形）与胶囊（圆角半透明面承载「活动行 + 数字」），由设置面板决定，也可切换' 80 1572 1560 14 (C 190 208 224 240)
Note '命中与拖拽：可拖拽区 = 内容外框 + 8 DIP 内边距；透明像素返回 HTTRANSPARENT，不挡壁纸操作；点击穿透开关开启后整块透传（从托盘恢复）' 80 1596 1560 13 (C 150 175 195 220)
Note '窗口按「展开态」一次性分配尺寸：收起时多余区域是透明像素、不参与命中 → 展开/收起不重分配，活动下拉列表也有空间可绘制' 80 1620 1560 13 (C 150 175 195 220)
Note '透明底限制：DirectWrite 只能灰度抗锯齿（ClearType 会产生彩边）；无新增依赖，exe 与内存增量约 2–4 MB（对比 WebView2 方案 100 MB+）' 80 1644 1560 13 (C 150 175 195 220)
Note '用户可自定义：4 个阶段色 + 字号 + 图形模式（简洁/条形/环形/隐藏文字）+ 形态；无皮肤文件、无图片资源、无字体打包' 80 1668 1560 13 (C 150 175 195 220)
FillR 0 1698 1680 44 0 (C 170 10 13 19)
FillR 24 1706 30 28 6 (C 200 90 140 190)
Txt '09:41' 1520 1698 130 44 17 (C 210 230 240 250) $YH 'Regular' 'Far'

$out = 'E:\Github\-Pomodoro-Timer\docs\widget-borderless.png'
$bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose()
"SAVED $out"