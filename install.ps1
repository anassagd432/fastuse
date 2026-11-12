# fastuse installer (Windows, no admin needed)
#
#   irm https://github.com/anassagd432/fastuse/releases/latest/download/install.ps1 | iex
#
# Downloads the release zip, verifies its SHA256 against SHA256SUMS, installs to
# %LOCALAPPDATA%\Programs\fastuse, adds it to User PATH, registers the browser native host
# + extension, and installs Antigravity skill & routing rules.

param(
    [string]$Repo = $(if ($env:FASTUSE_REPO) { $env:FASTUSE_REPO } else { "anassagd432/fastuse" }),
    [string]$Version = "latest",
    [switch]$NoAgy
)

$ErrorActionPreference = "Stop"

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

    # Ensure %LOCALAPPDATA%\Programs\fastuse is on User PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $segments = @()
    if ($userPath) { $segments = $userPath -split ";" | Where-Object { $_ -ne "" } }
    if ($segments -notcontains $dir) {
        $newPath = (@($dir) + $segments) -join ";"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-Host "==> added $dir to User PATH"
    }
    if (($env:Path -split ";") -notcontains $dir) {
        $env:Path = "$dir;$env:Path"
    }

    & $exe --version
    & $exe browser-install | Out-Host

    if (-not $NoAgy) {
        if (Get-Command agy -ErrorAction SilentlyContinue) {
            Write-Host "==> registering with Antigravity"
            agy mcp add fastuse $exe | Out-Host
        }
        foreach ($skillRoot in @(
            (Join-Path $env:USERPROFILE ".gemini\antigravity\global_skills\fastuse"),
            (Join-Path $env:USERPROFILE ".gemini\config\skills\fastuse")
        )) {
            New-Item -ItemType Directory -Force $skillRoot | Out-Null
            Copy-Item (Join-Path $dir "skills\fastuse\SKILL.md") (Join-Path $skillRoot "SKILL.md") -Force
        }
        $geminiMd = Join-Path $env:USERPROFILE ".gemini\config\GEMINI.md"
        New-Item -ItemType Directory -Force (Split-Path $geminiMd) | Out-Null
        $ruleBlock = @"

<!-- FASTUSE_COMPUTER_USE_START -->
## Default Computer Use & Browser Use Engine (`fastuse` MCP)
`fastuse` (MCP server `fastuse`) is Antigravity's **primary, default Computer Use and Browser Use tool** on both Windows and macOS.
The user never needs to mention `fastuse` by name. Automatically route requests as follows:
1. **Browser tasks ("access my Brave browser tab", "check my Chrome tab", "click/fill/navigate on the page", "open this site in my browser"):**
   - Immediately use MCP server `fastuse` tools `browser_state` and `browser_act`.
   - Do **not** launch Playwright, Puppeteer, or headless CDP browsers when the user asks to read or interact with their browser tab.
2. **Desktop / OS tasks ("do this on my computer", "operate <app>", "click/type/scroll in <window>", "what windows are open", "screenshot <window>"):**
   - Immediately use MCP server `fastuse` tools `act`, `get_window_state`, `list_windows`, and `screenshot`.
<!-- FASTUSE_COMPUTER_USE_END -->
"@
        $existing = if (Test-Path $geminiMd) { Get-Content $geminiMd -Raw } else { "" }
        if ($existing -notmatch "FASTUSE_COMPUTER_USE_START") {
            Add-Content -Path $geminiMd -Value $ruleBlock -Encoding UTF8
        }
        Write-Host "==> fastuse OS/browser skill and default routing rule installed for Antigravity"
    }

    Write-Host ""
    Write-Host "Installed fastuse to: $exe"
    Write-Host ""
    Write-Host "Next steps:"
    Write-Host "  1. Open a connected browser:  fastuse browser-open"
    Write-Host "  2. Verify health:             fastuse doctor"
    Write-Host "  3. Claude Code MCP add:       claude mcp add fastuse `"$exe`""
}
finally {
    Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
