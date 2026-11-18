<#
 Mide RAM/CPU del árbol de procesos de YouTube Mini (shell + msedgewebview2.exe hijos).
 Uso (PowerShell, con la app YA abierta):
   .\scripts\measure.ps1 -Label "mini-reproduciendo" -Minutes 30 -IntervalSec 5
 Salida: perf-<label>.csv (una fila por muestra) y un resumen por consola.
 Separa "shell" (youtube-mini.exe) de "webview" (msedgewebview2.exe del árbol) para no confundir
 el coste de la app con el coste del sitio/vídeo.
#>
param([string]$Label = "run", [int]$Minutes = 5, [int]$IntervalSec = 5, [string]$ExeName = "youtube-mini")

function Get-Tree($rootPid) {
  $all = Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, Name
  $ids = [System.Collections.Generic.HashSet[int]]::new(); [void]$ids.Add($rootPid)
  do { $n = $ids.Count
       foreach ($p in $all) { if ($ids.Contains([int]$p.ParentProcessId)) { [void]$ids.Add([int]$p.ProcessId) } }
  } while ($ids.Count -ne $n)
  $ids
}

$root = Get-Process $ExeName -ErrorAction Stop | Select-Object -First 1
$cores = [Environment]::ProcessorCount
$out = "perf-$Label.csv"
"t_sec,shell_ws_mb,shell_private_mb,webview_ws_mb,webview_private_mb,webview_procs,cpu_pct_total" | Set-Content $out
$prev = @{}; $t0 = Get-Date; $end = $t0.AddMinutes($Minutes)
while ((Get-Date) -lt $end) {
  $ids = Get-Tree $root.Id
  $procs = $ids | ForEach-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue } | Where-Object { $_ }
  $shell = $procs | Where-Object { $_.ProcessName -eq $ExeName }
  $web = $procs | Where-Object { $_.ProcessName -eq "msedgewebview2" }
  $cpuNow = ($procs | Measure-Object CPU -Sum).Sum; $now = Get-Date
  $cpuPct = if ($prev.t) { 100 * ($cpuNow - $prev.cpu) / (($now - $prev.t).TotalSeconds * $cores) } else { 0 }
  $prev = @{ t = $now; cpu = $cpuNow }
  $mb = { param($x, $prop) [math]::Round((($x | Measure-Object $prop -Sum).Sum) / 1MB, 1) }
  "{0},{1},{2},{3},{4},{5},{6}" -f [int]($now - $t0).TotalSeconds, (& $mb $shell WorkingSet64), (& $mb $shell PrivateMemorySize64),
     (& $mb $web WorkingSet64), (& $mb $web PrivateMemorySize64), @($web).Count, [math]::Round($cpuPct, 2) | Add-Content $out
  Start-Sleep -Seconds $IntervalSec
}
Import-Csv $out | Measure-Object shell_ws_mb, webview_ws_mb, cpu_pct_total -Average -Maximum | Format-Table Property, Average, Maximum
Write-Host "Guardado en $out. GPU: usa el Administrador de tareas > Detalles > columna 'GPU' o 'typeperf \"\GPU Engine(*)\Utilization Percentage\"'."
