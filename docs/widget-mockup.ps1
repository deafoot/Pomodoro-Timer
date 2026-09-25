Add-Type -AssemblyName System.Drawing
$ErrorActionPreference = 'Stop'
[System.Drawing.Text.TextRenderingHint] | Out-Null

$W = 1680; $H = 1290
$bmp = New-Object System.Drawing.Bitmap($W, $H)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = 'AntiAlias'
$g.TextRenderingHint = 'AntiAliasGridFit'
$g.InterpolationMode = 'HighQualityBicubic'

function C([int]$a,[int]$r,[int]$g2,[int]$b) { return [System.Drawing.Color]::FromArgb($a,$r,$g2,$b) }
function HexC([string]$hex) { $h = $hex.TrimStart('#'); return [System.Drawing.Color]::FromArgb(255, [Convert]::ToInt32($h.Substring(0,2),16), [Convert]::ToInt32($h.Substring(2,2),16), [Convert]::ToInt32($h.Substring(4,2),16)) }
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
function Shadow($x,$y,$w,$h,$r) { for ($i = 12; $i -ge 1; $i--) { $a = 5; FillR ($x-$i) ($y+$i*0.55) ($w+(2*$i)) ($h+(2*$i)) ($r+$i) (C $a 0 0 0) } }
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

$YH = 'Microsoft YaHei UI'

# ---- background ----
$bgRect = New-Object System.Drawing.Rectangle(0,0,$W,$H)
$lin = New-Object System.Drawing.Drawing2D.LinearGradientBrush($bgRect, (C 255 16 22 29), (C 255 34 48 62), 60)
$g.FillRectangle($lin, $bgRect); $lin.Dispose()
$glow = New-Object System.Drawing.Drawing2D.GraphicsPath
$glow.AddEllipse(-260, -420, 1100, 900)
$pg = New-Object System.Drawing.Drawing2D.PathGradientBrush($glow)
$pg.CenterColor = (C 26 90 140 190); $pg.SurroundColors = @((C 0 90 140 190))
$g.FillPath($pg, $glow); $pg.Dispose(); $glow.Dispose()

Txt '番茄钟 · 桌面小图标设计参考' 80 42 1200 44 34 (C 255 240 244 250) $YH 'Bold' 'Near'
Txt '尺寸单位 DIP（图中 1.2~1.4x 放大示意）  置顶 · 无边框 · 不进任务栏 · 透明背景  颜色全部取自当前主题（work / short / long / pause / surface / text）' 80 86 1500 26 17 (C 200 190 205 225) $YH 'Regular' 'Near'

function Panel($x,$y,$w,$h,$title,$sub) {
  FillR $x $y $w $h 14 (C 18 255 255 255)
  StrokeR $x $y $w $h 14 (C 26 255 255 255) 1
  Txt $title ($x+22) ($y+12) ($w-44) 26 19 (C 235 235 242 250) $YH 'Bold' 'Near'
  if ($sub) { Txt $sub ($x+22) ($y+38) ($w-44) 22 14 (C 150 190 205 225) $YH 'Regular' 'Near' }
}

$work = HexC '#e53e3e'; $shortC = HexC '#38a169'; $longC = HexC '#3182ce'; $pauseC = HexC '#718096'
$surfaceL = HexC '#ffffff'; $softL = HexC '#f0f0f3'; $borderL = HexC '#e3e3e8'; $textL = HexC '#1d1d1f'; $mutedL = HexC '#8b8b93'
$surfaceD = HexC '#232b3b'; $softD = HexC '#2d3648'; $borderD = HexC '#3b4557'; $textD = HexC '#f7fafc'; $mutedD = HexC '#9aa6b8'

function TW($text,$size,[string]$style) { $f = New-Object System.Drawing.Font($YH, $size, ([System.Drawing.FontStyle]$style), [System.Drawing.GraphicsUnit]::Pixel); $w = $g.MeasureString($text,$f).Width; $f.Dispose(); return $w }
function A([System.Drawing.Color]$c,[int]$alpha) { return [System.Drawing.Color]::FromArgb($alpha, $c.R, $c.G, $c.B) }

# ---------- 胶囊形态 ----------
function Capsule([single]$x,[single]$y,[single]$s,[System.Drawing.Color]$stage,[string]$activity,[string]$time,[string]$module,[bool]$paused,[bool]$dark,[bool]$pomoRunning,[bool]$showButtons = $true) {
  $fw = (280*$s); $pad = (10*$s); $headH = (32*$s); $timeH = (60*$s); $btnH = (32*$s); $gap = (8*$s); $rad = (18*$s)
  $fh = $pad + $headH + $gap + $timeH + $pad
  if ($showButtons) { $fh = $fh + $gap + $btnH }
  if ($dark) { $surface = $surfaceD; $soft = $softD; $border = $borderD; $text = $textD; $muted = $mutedD }
  else { $surface = $surfaceL; $soft = $softL; $border = $borderL; $text = $textL; $muted = $mutedL }
  Shadow $x $y $fw $fh $rad
  FillR $x $y $fw $fh $rad (A $surface 246)
  StrokeR $x $y $fw $fh $rad $border 1

  # 活动行
  $hy = $y + $pad
  $cy = $hy + $headH/2
  $g.FillEllipse((Brush $stage), ($x+$pad+(2*$s)), ($cy-(4*$s)), (8*$s), (8*$s))
  $btnSize = (26*$s)
  $openX = $x + $fw - $pad - $btnSize
  $editX = $openX - (6*$s) - $btnSize
  $textW = $editX - (6*$s) - ($x + $pad + (14*$s))
  Txt $activity ($x+$pad+(14*$s)) $hy $textW $headH (13*$s) $text $YH 'Bold' 'Near'
  Icon ([char]0xE70D) ($x+$pad+(14*$s)+$textW-(16*$s)) $hy (16*$s) $headH (9*$s) $muted 'Center'
  FillR $editX $hy $btnSize $headH (7*$s) $soft
  Icon ([char]0xE70F) $editX $hy $btnSize $headH (12*$s) $muted 'Center'
  FillR $openX $hy $btnSize $headH (7*$s) $soft
  Icon ([char]0xE8A7) $openX $hy $btnSize $headH (12*$s) $muted 'Center'

  # 时间
  $ty = $hy + $headH + $gap
  $tcolor = if ($paused) { $pauseC } else { $stage }
  Txt $time $x $ty $fw $timeH (38*$s) $tcolor $YH 'Bold' 'Center'

  if (-not $showButtons) { return $fh }

  # 按钮
  $by = $ty + $timeH + $gap
  $innerW = $fw - (2*$pad)
  $defs = @()
  if ($module -eq 'pomodoro') { $defs = @(@('E768','开始'),@('E71A','结束'),@('E72C','重置')) }
  else { $defs = @(@('E768','开始'),@('E71A','结束')) }
  $n = $defs.Count
  $bw = ($innerW - ($n-1)*(8*$s)) / $n
  for ($i = 0; $i -lt $n; $i++) {
    $bx = $x + $pad + $i*($bw + (8*$s))
    $label = $defs[$i][1]
    if ($i -eq 0) {
      if ($paused) { $label = '继续' }
      if ($pomoRunning) { $label = '暂停'; $defs[$i][0] = 'E769' }
      FillR $bx $by $bw $btnH (10*$s) $tcolor
      $iw = (15*$s); $gp = (5*$s); $lw = TW $label (12*$s) 'Bold'
      $sx = $bx + ($bw - ($iw + $gp + $lw))/2
      Icon ([char][Convert]::ToInt32($defs[$i][0],16)) $sx ($by+(1*$s)) $iw $btnH (11*$s) (C 255 255 255 255) 'Center'
      Txt $label ($sx+$iw+$gp) $by ($lw+8) $btnH (12*$s) (C 255 255 255 255) $YH 'Bold' 'Near'
    } else {
      $enabled = -not ($module -eq 'pomodoro' -and $i -eq 2 -and -not $pomoRunning)
      $fg = if ($enabled) { $text } else { (A $muted 110) }
      FillR $bx $by $bw $btnH (10*$s) (A $soft 220)
      $iw = (15*$s); $gp = (5*$s); $lw = TW $label (12*$s) 'Regular'
      $sx = $bx + ($bw - ($iw + $gp + $lw))/2
      Icon ([char][Convert]::ToInt32($defs[$i][0],16)) $sx ($by+(1*$s)) $iw $btnH (11*$s) $fg 'Center'
      Txt $label ($sx+$iw+$gp) $by ($lw+8) $btnH (12*$s) $fg $YH 'Regular' 'Near'
    }
  }
  return $fh
}

# ---------- 下拉列表 ----------
function DropList([single]$x,[single]$y,[single]$s,[string[]]$items,[int]$selected,[System.Drawing.Color]$stage) {
  $fw = (280*$s); $rowH = (28*$s); $pad = (5*$s)
  $fh = $items.Count*$rowH + (2*$pad)
  Shadow $x $y $fw $fh (12*$s)
  FillR $x $y $fw $fh (12*$s) (A $surfaceL 250)
  StrokeR $x $y $fw $fh (12*$s) $borderL 1
  for ($i = 0; $i -lt $items.Count; $i++) {
    $ry = $y + $pad + $i*$rowH
    if ($i -eq $selected) { FillR ($x+(4*$s)) $ry ($fw-(8*$s)) ($rowH-(2*$s)) (7*$s) (A $softL 255) }
    $fg = if ($i -eq $selected) { $stage } else { $textL }
    $st = if ($i -eq $selected) { 'Bold' } else { 'Regular' }
    Txt $items[$i] ($x+(10*$s)) $ry ($fw-(20*$s)) ($rowH-(2*$s)) (13*$s) $fg $YH $st 'Near'
  }
  return $fh
}

function Star([single]$cx,[single]$cy,[single]$ro,[single]$ri,[int]$points) {
  $p = New-Object System.Drawing.Drawing2D.GraphicsPath
  $list = New-Object 'System.Collections.Generic.List[System.Drawing.PointF]'
  for ($i = 0; $i -lt $points*2; $i++) {
    $ang = -[math]::PI/2 + $i*[math]::PI/$points
    $rad = if ($i % 2 -eq 0) { $ro } else { $ri }
    $list.Add((New-Object System.Drawing.PointF([single]($cx + $rad*[math]::Cos($ang)), [single]($cy + $rad*[math]::Sin($ang)))))
  }
  $p.AddPolygon($list.ToArray())
  return $p
}

# ---------- 拟物番茄 ----------
function Tomato([single]$cx,[single]$cy,[single]$s,[System.Drawing.Color]$stage,[single]$progress) {
  $r = (42*$s)
  $rr = $r + (12*$s)
  $tp = New-Object System.Drawing.Pen((A $stage 55), (7*$s))
  $g.DrawArc($tp, ($cx-$rr), ($cy-$rr), ($rr*2), ($rr*2), -90, 359.9)
  $tp.Dispose()
  $tp2 = New-Object System.Drawing.Pen($stage, (7*$s))
  $tp2.StartCap = 'Round'; $tp2.EndCap = 'Round'
  $g.DrawArc($tp2, ($cx-$rr), ($cy-$rr), ($rr*2), ($rr*2), -90, (360*$progress))
  $tp2.Dispose()

  $rect = New-Object System.Drawing.RectangleF(($cx-$r), ($cy-$r), ($r*2), ($r*2))
  $lg = New-Object System.Drawing.Drawing2D.LinearGradientBrush($rect, (HexC '#ff8a78'), (HexC '#bb2c24'), 55)
  $g.FillEllipse($lg, $rect); $lg.Dispose()
  $b = Brush (C 90 255 255 255)
  $g.FillEllipse($b, ($cx-$r*0.62), ($cy-$r*0.66), ($r*0.62), ($r*0.40)); $b.Dispose()
  $b2 = Brush (C 150 255 255 255)
  $g.FillEllipse($b2, ($cx-$r*0.52), ($cy-$r*0.56), ($r*0.26), ($r*0.18)); $b2.Dispose()

  $lp = Star $cx ($cy-$r*0.86) ($r*0.46) ($r*0.19) 5
  $lrect = New-Object System.Drawing.RectangleF(($cx-$r*0.5), ($cy-$r*1.4), ($r), ($r*0.7))
  $lg2 = New-Object System.Drawing.Drawing2D.LinearGradientBrush($lrect, (HexC '#74c977'), (HexC '#2c7a33'), 90)
  $g.FillPath($lg2, $lp); $lg2.Dispose()
  $g.DrawPath((New-Object System.Drawing.Pen((C 90 255 255 255), 1)), $lp)
  $lp.Dispose()
  StrokeR ($cx-(2.5*$s)) ($cy-$r*1.26) (5*$s) (10*$s) (2*$s) (HexC '#3f7d3f') 1
  FillR ($cx-(2.5*$s)) ($cy-$r*1.26) (5*$s) (10*$s) (2*$s) (HexC '#4a8c46')
}

# ================= 面板 =================
Panel 80 150 480 380 'A · 番茄钟模式' '3 按钮：开始 / 暂停 · 结束 · 重置'
$null = Capsule 152 220 1.2 $work '写代码' '24:12' 'pomodoro' $false $false $true
Txt '● 圆点与倒计时同色 = 当前阶段（专注红 / 短休绿 / 长休蓝）' 104 424 440 22 13 (C 190 205 220 235) $YH 'Regular' 'Near'
Txt '运行中主按钮显示「暂停」并使用阶段色；重置仅在空闲且满额时禁用' 104 448 440 22 13 (C 150 180 195 215) $YH 'Regular' 'Near'

Panel 600 150 480 380 'B · 计时模式' '2 按钮：开始 / 暂停 / 继续 · 结束'
$null = Capsule 672 220 1.2 $work '阅读' '12:34' 'timer' $false $false $false
Txt '模块切换时按钮组与动作一起切换，不会误触发另一个计时器' 624 424 440 22 13 (C 190 205 220 235) $YH 'Regular' 'Near'
Txt '「结束」写入历史（来源 = 计时），空闲时为「跳过」' 624 448 440 22 13 (C 150 180 195 215) $YH 'Regular' 'Near'

Panel 1120 150 480 380 'C · 拟物番茄形态（可切换）' '外环进度 · 状态色随阶段变化'
FillR 1160 206 400 34 17 (A (HexC '#ffffff') 42)
$g.FillEllipse((Brush $shortC), 1176, 219, 9, 9)
Txt '喝水' 1192 206 120 34 14 (C 240 245 250 255) $YH 'Bold' 'Near'
Icon ([char]0xE70D) 1300 206 16 34 9 (C 200 210 220 255) 'Center'
FillR 1380 210 26 26 7 (A (HexC '#ffffff') 40)
Icon ([char]0xE70F) 1380 210 26 26 12 (C 235 240 245 255) 'Center'
FillR 1412 210 26 26 7 (A (HexC '#ffffff') 40)
Icon ([char]0xE8A7) 1412 210 26 26 12 (C 235 240 245 255) 'Center'
Tomato 1360 302 1.25 $shortC 0.62
FillR 1264 372 192 34 17 (A (HexC '#ffffff') 235)
StrokeR 1264 372 192 34 17 (HexC '#e3e3e8') 1
$g.FillEllipse((Brush $shortC), (1280), 385, 8, 8)
Txt '短休息 04:38' (1296) 372 150 34 15 $shortC $YH 'Bold' 'Near'
$defsC = @(@('E768','开始'),@('E71A','结束'),@('E72C','重置'))
for ($i = 0; $i -lt 3; $i++) {
  $bx = 1200 + $i*109
  if ($i -eq 0) {
    $iw = 15; $gp = 5; $lw = TW '开始' 12 'Bold'
    $sx = $bx + (100 - ($iw + $gp + $lw))/2
    FillR $bx 424 100 30 9 $shortC
    Icon ([char]0xE768) $sx 425 $iw 30 11 (C 255 255 255 255) 'Center'
    Txt '开始' ($sx+$iw+$gp) 424 ($lw+8) 30 12 (C 255 255 255 255) $YH 'Bold' 'Near'
  } else {
    $iw = 15; $gp = 5; $lw = TW $defsC[$i][1] 12 'Regular'
    $sx = $bx + (100 - ($iw + $gp + $lw))/2
    FillR $bx 424 100 30 9 (A (HexC '#ffffff') 60)
    Icon ([char][Convert]::ToInt32($defsC[$i][0],16)) $sx 425 $iw 30 11 (C 235 240 245 255) 'Center'
    Txt $defsC[$i][1] ($sx+$iw+$gp) 424 ($lw+8) 30 12 (C 235 240 245 255) $YH 'Regular' 'Near'
  }
}
Txt '胶囊 / 拟物两种形态在「设置 · 桌面小图标」里切换，位置与尺寸各自记忆' 1144 470 440 22 13 (C 190 205 220 235) $YH 'Regular' 'Near'

Panel 80 560 620 610 'D · 活动选择（点 ▼ 展开）' '列表 = 事件管理里的事件，与主窗口完全一致'
$null = Capsule 130 640 1.0 $work '写代码' '24:12' 'pomodoro' $false $false $true
$null = DropList 130 806 1.0 @('写代码','阅读','写作','学习','游戏','散步','喝水','听音乐','午休') 0 $work
Txt '选中项用阶段色高亮；列表超过 9 行时可用滚轮滚动' 104 1086 560 22 13 (C 190 205 220 235) $YH 'Regular' 'Near'
Txt '活动只影响记录名与方案映射，不改动计时时间' 104 1110 560 22 13 (C 150 180 195 215) $YH 'Regular' 'Near'

Panel 740 560 440 610 'E · 活动输入态（点 ✎）' '回车确认 · Esc 取消 · 点别处确认'
FillR 770 636 392 62 18 (A $surfaceL 250)
StrokeR 770 636 392 62 18 $borderL 1
$g.FillEllipse((Brush $work), 790, 663, 11, 11)
Txt '写代' 812 636 200 62 19 $textL $YH 'Bold' 'Near'
FillR 880 650 3 34 1 $work
Icon ([char]0xE70D) 1112 636 20 62 12 $mutedL 'Center'
Txt 'shu' 812 702 120 24 17 (C 190 145 150 160) $YH 'Regular' 'Near'
$penU = New-Object System.Drawing.Pen((C 130 130 140 160), 1.5)
$penU.DashStyle = 'Dot'
$g.DrawLine($penU, 812, 728, 862, 728); $penU.Dispose()
FillR 812 734 30 3 1.5 (A $work 255)
Txt '拼音未上屏时在输入框内绘制（GCS_COMPSTR），窗口用 ImmSetCompositionWindow 定位' 770 748 400 22 12 (C 160 185 200 225) $YH 'Regular' 'Near'
Txt '候选条' 770 776 400 22 12 (C 130 155 175 205) $YH 'Regular' 'Near'
FillR 770 800 300 34 10 (A $surfaceL 250)
StrokeR 770 800 300 34 10 $borderL 1
Txt '1 输入    2 输出    3 输 入' 786 800 280 34 15 $textL $YH 'Regular' 'Near'
Txt '编辑期间窗口临时激活（否则收不到键盘/输入法），结束编辑立刻恢复不抢焦点' 770 866 400 44 13 (C 190 205 220 235) $YH 'Regular' 'Near'
Txt '输入的新名称自动加入事件列表；与主窗口共用同一套编辑状态机' 770 916 400 22 13 (C 150 180 195 215) $YH 'Regular' 'Near'
Txt '编辑提交流程' 770 968 400 22 12 (C 130 155 175 205) $YH 'Regular' 'Near'
$steps = @('点 ✎ 进入编辑','拼音上屏 / 输入','回车 → 写入事件列表')
for ($i = 0; $i -lt 3; $i++) {
  FillR (770+$i*136) 998 120 32 9 (A (HexC '#ffffff') 38)
  Txt $steps[$i] (770+$i*136) 998 120 32 12 (C 220 232 240 255) $YH 'Regular' 'Center'
  if ($i -lt 2) { Icon ([char]0xE76C) (770+$i*136+120) 998 16 32 10 (C 150 175 195 255) 'Center' }
}

Panel 1200 560 400 610 'F · 阶段配色与暗色' '颜色全部取自当前主题'
$mini = @(
  @(1225,640,'24:12',$work,$false,'专注 #e53e3e'),
  @(1425,640,'04:38',$shortC,$false,'短休息 #38a169'),
  @(1225,790,'14:02',$longC,$false,'长休息 #3182ce'),
  @(1425,790,'24:12',$work,$true,'暗色主题自动跟随')
)
foreach ($m in $mini) {
  $null = Capsule $m[0] $m[1] 0.62 $m[3] '写代码' $m[2] 'pomodoro' $false $m[4] $false $false
  Txt $m[5] $m[0] ($m[1]+82) 180 22 12 (C 200 215 230 240) $YH 'Regular' 'Near'
}
Txt '暂停时倒计时与主按钮转 pause 色 #718096' 1225 920 360 22 13 (C 190 205 220 235) $YH 'Regular' 'Near'
Txt '活动行圆点同色，不靠文字也能看出阶段' 1225 944 360 22 13 (C 190 205 220 235) $YH 'Regular' 'Near'
Txt '主题/暗色/自定义配色切换后小图标同步刷新' 1225 968 360 22 13 (C 190 205 220 235) $YH 'Regular' 'Near'
Txt '空闲未开始：显示 00:00 与「开始」，圆点用当前阶段色' 1225 992 360 44 13 (C 150 180 195 215) $YH 'Regular' 'Near'
Txt '尺寸档位（150 / 200 / 260）只缩放整体，控件比例不变' 1225 1036 360 44 13 (C 150 180 195 215) $YH 'Regular' 'Near'

# ---- 底部：任务栏示意 + 说明 ----
FillR 0 1246 1680 44 0 (C 170 10 13 19)
FillR 24 1254 30 28 6 (C 200 90 140 190)
for ($i = 0; $i -lt 4; $i++) { FillR (68 + $i*40) 1256 30 24 5 (C 90 255 255 255) }
Txt '09:41' 1520 1246 130 44 17 (C 210 230 240 250) $YH 'Regular' 'Far'
Txt '布局（DIP）：总宽 280 · 活动行 32 · 时间区 60 · 按钮 32 · 内边距 10 · 圆角 18 · 按钮圆角 10 · 图标 26' 80 1186 1520 22 14 (C 175 195 210 235) $YH 'Regular' 'Near'
Txt '字体 Microsoft YaHei UI（时间字号 = min(设置字号, 40)）  ·  图标 Segoe Fluent Icons  ·  按钮动作复用现有 Action，无新增业务逻辑' 80 1210 1520 22 14 (C 140 165 185 215) $YH 'Regular' 'Near'

$out = 'E:\Github\-Pomodoro-Timer\docs\widget-mockup.png'
$bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose()
"SAVED $out"

