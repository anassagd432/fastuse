# fastuse installer (Windows, no admin needed)
#
#   irm https://github.com/OWNER/fastuse/releases/latest/download/install.ps1 | iex
#
# Downloads the release zip, verifies its SHA256 against the release's SHA256SUMS, installs to
# %LOCALAPPDATA%\Programs\fastuse, and registers the server with Antigravity (agy) if it is installed.
# It does not touch PATH, does not add startup tasks, and does not edit permission settings.

param(
    [string]$Repo = $(if ($env:FASTUSE_REPO) { $env:FASTUSE_REPO } else { "OWNER/fastuse" }),
    [string]$Version = "latest",
    [switch]$NoAgy
)

$ErrorActionPreference = "Stop"
if ($Repo -like "OWNER/*") { throw "Set the repository: -Repo <user>/fastuse (or `$env:FASTUSE_REPO)." }

$base = if ($Version -eq "latest") { "https://github.com/$Repo/releases/latest/download" } else { "https://github.com/$Repo/releases/download/$Version" }
$zipName = "fastuse-windows-x86_64.zip"
$dir = Join-Path $env:LOCALAPPDATA "Programs\fastuse"
$tmp = Join-Path ([IO.Path]::GetTempPath()) ("fastuse-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force $tmp | Out-Null
$ProgressPreference = "SilentlyContinue"

try {
    Write-Host "==> downloading $zipName from $Repo ($Version)"
    Invoke-WebRequest "$base/$zipName" -OutFile (Join-Path $tmp $zipName) -UseBasicParsing
    Invoke-WebRequest "$base/SHA256SUMS" -OutFile (Join-Path $tmp "SHA256SUMS") -UseBasicParsing

    $want = (Get-Content (Join-Path $tmp "SHA256SUMS") | Where-Object { $_ -match [regex]::Escape($zipName) } | Select-Object -First 1) -split "\s+" | Select-Object -First 1
    $got = (Get-FileHash (Join-Path $tmp $zipName) -Algorithm SHA256).Hash
    if (-not $want -or $want.ToLower() -ne $got.ToLower()) { throw "SHA256 mismatch for $zipName (expected '$want', got '$got'). Refusing to install." }
    Write-Host "==> checksum verified"

    $stage = Join-Path $tmp "x"
    Expand-Archive (Join-Path $tmp $zipName) -DestinationPath $stage -Force
    New-Item -ItemType Directory -Force $dir | Out-Null
    Copy-Item (Join-Path $stage "*") $dir -Recurse -Force
    $exe = Join-Path $dir "fastuse.exe"
    & $exe --version

    if (-not $NoAgy -and (Get-Command agy -ErrorAction SilentlyContinue)) {
        Write-Host "==> registering with Antigravity"
        agy mcp add fastuse $exe | Out-Host
        $skills = Join-Path $env:USERPROFILE ".gemini\antigravity\global_skills\fastuse"
        New-Item -ItemType Directory -Force $skills | Out-Null
        Copy-Item (Join-Path $dir "skills\fastuse\SKILL.md") (Join-Path $skills "SKILL.md") -Force
        Write-Host "==> skill installed to $skills"
    } elseif (-not $NoAgy) {
        Write-Host "agy not found; skipped Antigravity registration. Later: agy mcp add fastuse `"$exe`""
    }
    Write-Host ""
    Write-Host "Installed to $dir"
    Write-Host "Headless agy (-p / stream-json) auto-denies MCP calls unless you allow them. See the README, section 'Permissions'."
}
finally {
    Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
