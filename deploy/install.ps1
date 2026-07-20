# NovaOS one-command installer for Windows.
#
#   irm https://raw.githubusercontent.com/bhouvana/NovaOS/master/deploy/install.ps1 | iex
#
# Detects whether Docker Desktop is installed, installs it via winget if not,
# pulls the prebuilt NovaOS image, runs it, and opens your browser to the
# desktop. No Docker knowledge required.
#
# One honest limitation: Windows requires Docker Desktop, which needs WSL2 and
# sometimes a restart the very first time it's set up on a machine - that one
# step can't be skipped by any script, Microsoft/Docker require it themselves.
# If that happens, this script tells you exactly what to do and you just run
# the same command again afterward.

$ErrorActionPreference = "Stop"
$Image = "ghcr.io/bhouvana/novaos:latest"
$PortEnv = $env:NOVAOS_PORT
if ([string]::IsNullOrEmpty($PortEnv)) { $Port = 8080 } else { $Port = [int]$PortEnv }
$Name = "novaos"

function Info($msg)  { Write-Host "==> $msg" -ForegroundColor Cyan }
function Ok($msg)    { Write-Host "[OK] $msg" -ForegroundColor Green }
function Warn($msg)  { Write-Host "[!] $msg" -ForegroundColor Yellow }
function Die($msg)   { Write-Host "[X] $msg" -ForegroundColor Red; exit 1 }

# --- 1. make sure Docker is installed and running ---------------------------
$dockerCmd = Get-Command docker -ErrorAction SilentlyContinue
if (-not $dockerCmd) {
    Warn "Docker isn't installed - installing Docker Desktop now."
    $winget = Get-Command winget -ErrorAction SilentlyContinue
    if (-not $winget) {
        Warn "winget isn't available on this system (needs Windows 10 1809+ / Windows 11)."
        Start-Process "https://www.docker.com/products/docker-desktop/"
        Die "Install Docker Desktop from the page that just opened, then run this command again."
    }
    Info "Installing via winget (you may see a Windows permission prompt - accept it)..."
    winget install -e --id Docker.DockerDesktop --accept-package-agreements --accept-source-agreements

    Warn "Docker Desktop is installed. On a fresh machine it usually needs WSL2 enabled and ONE restart before it can run - this is required by Docker/Microsoft, no script can skip it."
    Info "Trying to start Docker Desktop..."
    $dockerDesktopPath = "$Env:ProgramFiles\Docker\Docker\Docker Desktop.exe"
    if (Test-Path $dockerDesktopPath) {
        Start-Process $dockerDesktopPath
    }
    Info "Waiting for Docker to become ready (up to 3 minutes)..."
    $ready = $false
    for ($i = 0; $i -lt 90; $i++) {
        Start-Sleep -Seconds 2
        docker info *> $null
        if ($LASTEXITCODE -eq 0) { $ready = $true; break }
    }
    if (-not $ready) {
        Die "Docker isn't ready yet. If Windows asked you to restart or enable WSL2, do that now, then just run this same command again - it'll pick up right where it left off."
    }
} else {
    Ok "Docker is already installed."
}

docker info *> $null
if ($LASTEXITCODE -ne 0) {
    Die "Docker is installed but not running. Start Docker Desktop from the Start menu, wait for it to say 'running', then re-run this command."
}
Ok "Docker is running."

# --- 2. get the image --------------------------------------------------------
Info "Pulling the NovaOS image (this is a one-time download, a few GB)..."
docker pull $Image *> $null
if ($LASTEXITCODE -ne 0) {
    Warn "Couldn't pull the prebuilt image (not published yet, or offline) - building it locally instead. This is much slower (~20-30 minutes, one time only) since it compiles the whole desktop from source."
    $git = Get-Command git -ErrorAction SilentlyContinue
    if (-not $git) { Die "git is required to build locally. Install git (or wait for the prebuilt image to be published) and re-run." }
    $tmp = Join-Path $env:TEMP "novaos-build-$(Get-Random)"
    git clone --depth 1 https://github.com/bhouvana/NovaOS.git $tmp
    Push-Location $tmp
    docker build -t novaos .
    Pop-Location
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    $Image = "novaos"
}
Ok "NovaOS image ready."

# --- 3. run it ----------------------------------------------------------------
# The two named volumes are what make this a real second OS instead of a demo
# that forgets everything on restart: your files/settings and anything
# installed via the in-desktop Software Center persist across restarts and
# even NovaOS image updates (everything else always comes from the image, so
# a newer NovaOS pull still gets you the update, not a frozen copy).
docker rm -f $Name *> $null
Info "Starting NovaOS..."
docker run -d --name $Name --restart unless-stopped -p "${Port}:8080" -e PORT=8080 --privileged `
  -v novaos-home:/opt/novaos/tc-root/root `
  -v novaos-tce:/opt/novaos/tc-root/etc/sysconfig/tcedir `
  $Image *> $null

Info "Waiting for the desktop to come up..."
$url = "http://localhost:$Port/"
$up = $false
for ($i = 0; $i -lt 60; $i++) {
    try {
        Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 2 *> $null
        $up = $true
        break
    } catch { Start-Sleep -Seconds 1 }
}
if (-not $up) { Warn "Taking longer than expected - check 'docker logs $Name' if this doesn't load." }

Ok "NovaOS is running at $url"
Start-Process $url
Write-Host ""
Write-Host "If your browser didn't open automatically, go to: $url"
