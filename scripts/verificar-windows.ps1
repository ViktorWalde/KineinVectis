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
    [switch]$Foto,
    [switch]$Linux,
    [string]$Distro = 'FedoraLinux-44'
)

$ErrorActionPreference = 'Continue'
$raiz = Split-Path -Parent $PSScriptRoot
Set-Location $raiz
$falhas = New-Object System.Collections.Generic.List[string]

function Passo([string]$nome, [scriptblock]$acao) {
    Write-Host "== $nome =="
    $global:LASTEXITCODE = 0
    & $acao
    if ($LASTEXITCODE -ne 0) {
        $script:falhas.Add($nome)
        Write-Host "FALHOU: $nome (codigo $LASTEXITCODE)" -ForegroundColor Red
    } else {
        Write-Host 'ok' -ForegroundColor Green
    }
}

function Falha([string]$motivo) {
    Write-Host $motivo -ForegroundColor Red
    $global:LASTEXITCODE = 1
}

# --- ambiente ---------------------------------------------------------------
# O Visual Studio e' achado pelo vswhere, que mora fora do PATH; o
# Launch-VsDevShell.ps1 o chama pelo nome, por isso a pasta dele entra antes.
$instalador = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer'
$vswhere = Join-Path $instalador 'vswhere.exe'
$vs = $null
if (Test-Path $vswhere) {
    $vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
}
Passo 'ambiente' {
    if (-not $vs) { Falha 'Visual Studio com o "Desenvolvimento para desktop com C++" nao encontrado.'; return }
    $env:Path = "$instalador;$env:Path"
    & (Join-Path $vs 'Common7\Tools\Launch-VsDevShell.ps1') -Arch amd64 -HostArch amd64 -SkipAutomaticLocation | Out-Null
    Set-Location $raiz
    if (-not $env:CMAKE_PREFIX_PATH -or -not (Test-Path (Join-Path $env:CMAKE_PREFIX_PATH 'bin\qtpaths.exe'))) {
        Falha 'CMAKE_PREFIX_PATH deve apontar para o Qt MSVC (ex.: C:\Qt\6.12.0\msvc2022_64).'; return
    }
    foreach ($ferramenta in 'cl', 'cmake', 'ninja', 'cargo', 'python', 'wsl') {
        if (-not (Get-Command $ferramenta -ErrorAction SilentlyContinue)) { Falha "ferramenta ausente: $ferramenta"; return }
    }
    $global:LASTEXITCODE = 0
}
if ($falhas.Count -gt 0) { Write-Host 'Sem ambiente, o resto mediria a maquina e nao o codigo.' -ForegroundColor Red; exit 1 }

$qtBin = Join-Path $env:CMAKE_PREFIX_PATH 'bin'
$clDir = Split-Path (Get-Command cl).Source
$build = Join-Path $raiz 'build\windows-msvc-debug'

# --- rust ---------------------------------------------------------------------
Passo 'cargo fmt --all --check' { cargo fmt --all --check }
Passo 'cargo clippy --workspace --all-targets --all-features -- -D warnings' {
    cargo clippy --workspace --all-targets --all-features -- -D warnings
}
Passo 'cargo test --workspace --all-features --no-fail-fast' {
    cargo test --workspace --all-features --no-fail-fast
}

# --- c++ ----------------------------------------------------------------------
Passo 'cmake --preset windows-msvc-debug' { cmake --preset windows-msvc-debug | Out-Null }
Passo 'cmake --build --preset windows-msvc-debug' { cmake --build --preset windows-msvc-debug }
Passo 'ctest' { ctest --test-dir $build --output-on-failure --timeout 120 }

# --- smoke --------------------------------------------------------------------
# O executavel acha as DLLs do Qt e o runtime do ASan (ao lado do cl) pelo PATH.
function Abrir-Ide([hashtable]$ambiente, [string[]]$argumentos) {
    $antes = @{}
    foreach ($chave in $ambiente.Keys) {
        $antes[$chave] = [Environment]::GetEnvironmentVariable($chave)
        [Environment]::SetEnvironmentVariable($chave, $ambiente[$chave])
    }
    $exe = Join-Path $build 'ui\kinein-vectis.exe'
    if ($argumentos) {
        $processo = Start-Process -FilePath $exe -ArgumentList $argumentos -WorkingDirectory $raiz -PassThru
    } else {
        $processo = Start-Process -FilePath $exe -WorkingDirectory $raiz -PassThru
    }
    foreach ($chave in $antes.Keys) { [Environment]::SetEnvironmentVariable($chave, $antes[$chave]) }
    return $processo
}

Passo 'cargo build -p kinein-core -p kinein-adapter-sqlite' { cargo build -p kinein-core -p kinein-adapter-sqlite }
# O core sobe depois do QML: no Debug com ASan isso levou 52 s de CPU (medido
# em 2026-10-09). O smoke espera o core ate' 120 s, olhando a cada meio segundo:
# ele mede "subiu", nao "subiu rapido".
Passo 'smoke (a IDE sobe, o core e filho, nada fica orfao)' {
    $caminho = "$qtBin;$clDir;$env:Path"
    $ide = Abrir-Ide @{ Path = $caminho; QT_QPA_PLATFORM = 'offscreen' } @()
    $relogio = [Diagnostics.Stopwatch]::StartNew()
    $filhos = @()
    while ($relogio.Elapsed.TotalSeconds -lt 120 -and -not $ide.HasExited -and $filhos.Count -eq 0) {
        Start-Sleep -Milliseconds 500
        $filhos = @(Get-CimInstance Win32_Process -Filter "Name='kinein-core.exe'" | Where-Object { $_.ParentProcessId -eq $ide.Id })
    }
    if ($ide.HasExited) { Falha "a IDE saiu sozinha (codigo $($ide.ExitCode))"; return }
    Write-Host "core subiu em $([int]$relogio.Elapsed.TotalSeconds) s"
    Stop-Process -Id $ide.Id -Force
    Start-Sleep -Seconds 2
    if ($filhos.Count -eq 0) { Falha 'o kinein-core.exe nao subiu como filho da IDE'; return }
    $orfaos = @(Get-Process -Id ($filhos | ForEach-Object { $_.ProcessId }) -ErrorAction SilentlyContinue)
    if ($orfaos.Count -gt 0) { Falha 'o kinein-core.exe ficou orfao depois de a IDE fechar'; return }
    $global:LASTEXITCODE = 0
}

if ($Foto) {
    Passo 'foto (a janela real, pela propria IDE)' {
        $png = Join-Path $build 'foto-windows.png'
        Remove-Item $png -ErrorAction SilentlyContinue
        $ide = Abrir-Ide @{
            Path = "$qtBin;$clDir;$env:Path"; KINEIN_SCREENSHOT = $png; KINEIN_SCREENSHOT_DELAY_MS = '7000'
            KINEIN_SCREENSHOT_SIZE = '1600x1000'; KINEIN_PERF_EXIT = '1'
        } @($raiz)
        if (-not $ide.WaitForExit(180000)) { Stop-Process -Id $ide.Id -Force; Falha 'a foto nao saiu em 180 s'; return }
        if (-not (Test-Path $png)) { Falha 'a IDE saiu sem gravar a foto'; return }
        Write-Host "foto: $png"
        $global:LASTEXITCODE = 0
    }
}

# --- agnostico e Linux (WSL) --------------------------------------------------
$raizWsl = (wsl -d $Distro -- wslpath -a $raiz.Replace('\', '/')) | Select-Object -First 1
Passo "agnostico (WSL $Distro)" { wsl -d $Distro --cd $raizWsl -- bash scripts/verificar-agnostico.sh }
if ($Linux) {
    Passo "gate do Linux (espelho no WSL $Distro)" { wsl -d $Distro --cd $raizWsl -- bash scripts/espelhar-no-wsl.sh }
}

if ($falhas.Count -gt 0) {
    Write-Host ''
    Write-Host 'FALHOU em:' -ForegroundColor Red
    $falhas | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    exit 1
}
Write-Host 'gate do Windows: tudo verde' -ForegroundColor Green
