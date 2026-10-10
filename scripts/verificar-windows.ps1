<#
.SYNOPSIS
O gate do Windows (DocsPublic/roadmaps/60 3.1, decisao D8; o roteiro do 3.4).

.DESCRIPTION
POR QUE ESTE SCRIPT EXISTE (2026-10-09). O desenvolvimento passou para o
Windows 11 Pro, e o autor pediu um porte "impecavel": o Windows e' par do
Linux. Este script roda NATIVO no Windows tudo o que depende do sistema e, no
fim, chama o scripts/verificar-agnostico.sh dentro do WSL para o que nao
depende. Com -Linux, roda tambem o gate completo do Linux (verificar.sh) num
espelho do checkout dentro do WSL (scripts/espelhar-no-wsl.sh).

O que roda aqui, nesta ordem, e por que:
  ambiente   Visual Studio (cl, CMake, Ninja), Rust, Qt, Python e WSL presentes;
             sem um deles o resto mediria a maquina, nao o codigo.
  rust       cargo fmt --check, clippy -D warnings e cargo test, como no Linux.
  c++        o preset windows-msvc-debug: MSVC estrito (/W4 /WX /sdl) com ASan.
  ctest      os testes C++ da UI.
  smoke      a IDE sobe sem janela, o core sobe como filho e nao sobra orfao.
  foto       (-Foto) a janela real fotografada pela propria IDE.
  agnostico  docs, links, mapa, catraca, shellcheck, QML e fiacao, no WSL.

O arquivo e' ASCII de proposito: o PowerShell 5.1 le .ps1 sem BOM como ANSI, e
um acento aqui viraria lixo (o que corrompeu o lib.rs em 2026-10-09).

.EXAMPLE
scripts\verificar-windows.ps1
.EXAMPLE
scripts\verificar-windows.ps1 -Foto -Linux
#>
param(
    # -Foto e' o nome do projeto para a captura da tela (contribuindo/01).
    [Alias('Foto')][switch]$Screenshot,
    [switch]$Linux,
    [string]$Distro = 'FedoraLinux-44'
)

$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
$failures = New-Object System.Collections.Generic.List[string]

function Step([string]$name, [scriptblock]$action) {
    Write-Host "== $name =="
    $global:LASTEXITCODE = 0
    & $action
    if ($LASTEXITCODE -ne 0) {
        $script:failures.Add($name)
        Write-Host "FALHOU: $name (codigo $LASTEXITCODE)" -ForegroundColor Red
    } else {
        Write-Host 'ok' -ForegroundColor Green
    }
}

function Fail([string]$reason) {
    Write-Host $reason -ForegroundColor Red
    $global:LASTEXITCODE = 1
}

# --- ambiente ---------------------------------------------------------------
# O Visual Studio e' achado pelo vswhere, que mora fora do PATH; o
# Launch-VsDevShell.ps1 o chama pelo nome, por isso a pasta dele entra antes.
$installer = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer'
$vswhere = Join-Path $installer 'vswhere.exe'
$vs = $null
if (Test-Path $vswhere) {
    $vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
}
Step 'ambiente' {
    if (-not $vs) { Fail 'Visual Studio com o "Desenvolvimento para desktop com C++" nao encontrado.'; return }
    $env:Path = "$installer;$env:Path"
    & (Join-Path $vs 'Common7\Tools\Launch-VsDevShell.ps1') -Arch amd64 -HostArch amd64 -SkipAutomaticLocation | Out-Null
    Set-Location $root
    if (-not $env:CMAKE_PREFIX_PATH -or -not (Test-Path (Join-Path $env:CMAKE_PREFIX_PATH 'bin\qtpaths.exe'))) {
        Fail 'CMAKE_PREFIX_PATH deve apontar para o Qt MSVC (ex.: C:\Qt\6.12.0\msvc2022_64).'; return
    }
    foreach ($tool in 'cl', 'cmake', 'ninja', 'cargo', 'python', 'wsl') {
        if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) { Fail "ferramenta ausente: $tool"; return }
    }
    $global:LASTEXITCODE = 0
}
if ($failures.Count -gt 0) { Write-Host 'Sem ambiente, o resto mediria a maquina e nao o codigo.' -ForegroundColor Red; exit 1 }

$qtBin = Join-Path $env:CMAKE_PREFIX_PATH 'bin'
$clDir = Split-Path (Get-Command cl).Source
$build = Join-Path $root 'build\windows-msvc-debug'

# --- rust ---------------------------------------------------------------------
Step 'cargo fmt --all --check' { cargo fmt --all --check }
Step 'cargo clippy --workspace --all-targets --all-features -- -D warnings' {
    cargo clippy --workspace --all-targets --all-features -- -D warnings
}
Step 'cargo test --workspace --all-features --no-fail-fast' {
    cargo test --workspace --all-features --no-fail-fast
}

# --- c++ ----------------------------------------------------------------------
Step 'cmake --preset windows-msvc-debug' { cmake --preset windows-msvc-debug | Out-Null }
Step 'cmake --build --preset windows-msvc-debug' { cmake --build --preset windows-msvc-debug }
Step 'ctest' { ctest --test-dir $build --output-on-failure --timeout 120 }

# --- smoke --------------------------------------------------------------------
# O executavel acha as DLLs do Qt e o runtime do ASan (ao lado do cl) pelo PATH.
function Start-Ide([hashtable]$environment, [string[]]$arguments) {
    $previous = @{}
    foreach ($key in $environment.Keys) {
        $previous[$key] = [Environment]::GetEnvironmentVariable($key)
        [Environment]::SetEnvironmentVariable($key, $environment[$key])
    }
    $exe = Join-Path $build 'ui\kinein-vectis.exe'
    if ($arguments) {
        $process = Start-Process -FilePath $exe -ArgumentList $arguments -WorkingDirectory $root -PassThru
    } else {
        $process = Start-Process -FilePath $exe -WorkingDirectory $root -PassThru
    }
    foreach ($key in $previous.Keys) { [Environment]::SetEnvironmentVariable($key, $previous[$key]) }
    return $process
}

Step 'cargo build -p kinein-core -p kinein-adapter-sqlite' { cargo build -p kinein-core -p kinein-adapter-sqlite }
# O core sobe depois do QML: no Debug com ASan isso levou 52 s de CPU (medido
# em 2026-10-09). O smoke espera o core ate' 120 s, olhando a cada meio segundo:
# ele mede "subiu", nao "subiu rapido".
Step 'smoke (a IDE sobe, o core e filho, nada fica orfao)' {
    $searchPath = "$qtBin;$clDir;$env:Path"
    $ide = Start-Ide @{ Path = $searchPath; QT_QPA_PLATFORM = 'offscreen' } @()
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $children = @()
    while ($clock.Elapsed.TotalSeconds -lt 120 -and -not $ide.HasExited -and $children.Count -eq 0) {
        Start-Sleep -Milliseconds 500
        $children = @(Get-CimInstance Win32_Process -Filter "Name='kinein-core.exe'" | Where-Object { $_.ParentProcessId -eq $ide.Id })
    }
    if ($ide.HasExited) { Fail "a IDE saiu sozinha (codigo $($ide.ExitCode))"; return }
    Write-Host "core subiu em $([int]$clock.Elapsed.TotalSeconds) s"
    Stop-Process -Id $ide.Id -Force
    Start-Sleep -Seconds 2
    if ($children.Count -eq 0) { Fail 'o kinein-core.exe nao subiu como filho da IDE'; return }
    $orphans = @(Get-Process -Id ($children | ForEach-Object { $_.ProcessId }) -ErrorAction SilentlyContinue)
    if ($orphans.Count -gt 0) { Fail 'o kinein-core.exe ficou orfao depois de a IDE fechar'; return }
    $global:LASTEXITCODE = 0
}

if ($Screenshot) {
    Step 'foto (a janela real, pela propria IDE)' {
        $png = Join-Path $build 'foto-windows.png'
        Remove-Item $png -ErrorAction SilentlyContinue
        $ide = Start-Ide @{
            Path = "$qtBin;$clDir;$env:Path"; KINEIN_SCREENSHOT = $png; KINEIN_SCREENSHOT_DELAY_MS = '7000'
            KINEIN_SCREENSHOT_SIZE = '1600x1000'; KINEIN_PERF_EXIT = '1'
        } @($root)
        if (-not $ide.WaitForExit(180000)) { Stop-Process -Id $ide.Id -Force; Fail 'a foto nao saiu em 180 s'; return }
        if (-not (Test-Path $png)) { Fail 'a IDE saiu sem gravar a foto'; return }
        Write-Host "foto: $png"
        $global:LASTEXITCODE = 0
    }
}

# --- agnostico e Linux (WSL) --------------------------------------------------
$wslRoot = (wsl -d $Distro -- wslpath -a $root.Replace('\', '/')) | Select-Object -First 1
Step "agnostico (WSL $Distro)" { wsl -d $Distro --cd $wslRoot -- bash scripts/verificar-agnostico.sh }
if ($Linux) {
    Step "gate do Linux (espelho no WSL $Distro)" { wsl -d $Distro --cd $wslRoot -- bash scripts/espelhar-no-wsl.sh }
}

if ($failures.Count -gt 0) {
    Write-Host ''
    Write-Host 'FALHOU em:' -ForegroundColor Red
    $failures | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    exit 1
}
Write-Host 'gate do Windows: tudo verde' -ForegroundColor Green
