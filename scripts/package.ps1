# Build and package MalodyStore as a self-contained app image (Windows).
# Requires JDK 17+ with jlink and jpackage on PATH.
# Output: build/dist/MalodyStore/
param(
    [string]$AppName = "MalodyStore",
    [string]$AppVersion = "0.0.1"
)

$ErrorActionPreference = "Stop"
$javaModules = "java.base,java.compiler,java.desktop,java.instrument,java.logging,java.management,java.naming,java.net.http,java.security.jgss,java.sql,java.xml,jdk.crypto.ec,jdk.unsupported,jdk.zipfs"

# Resolve to absolute paths (Windows jpackage handles backslashes more reliably)
$buildDir = Join-Path $PWD "build"
$libsDir = Join-Path $buildDir "libs"
$jlinkDir = Join-Path $buildDir "jlink-runtime"
$distDir = Join-Path $buildDir "dist"
$appDir = Join-Path $distDir $AppName

Write-Host "=== Step 1: Build bootJar ==="
./gradlew bootJar --no-daemon -q

$jarFile = Get-ChildItem "$libsDir/*.jar" | Select-Object -First 1
if (-not $jarFile) { throw "bootJar not found in $libsDir" }
Write-Host "  JAR: $($jarFile.Name)"

Write-Host "=== Step 2: jlink (stripped JRE) ==="
jlink `
  --output "$jlinkDir" `
  --add-modules $javaModules `
  --strip-debug --no-header-files --no-man-pages `
  --compress 2
if ($LASTEXITCODE -ne 0) { throw "jlink failed" }
Write-Host "  Runtime: $((Get-ChildItem $jlinkDir -Recurse | Measure-Object).Count) files"

Write-Host "=== Step 3: jpackage (app image) ==="
jpackage `
  --type app-image `
  --name $AppName `
  --app-version $AppVersion `
  --input "$libsDir" `
  --main-jar $jarFile.Name `
  --main-class org.springframework.boot.loader.launch.JarLauncher `
  --dest "$distDir" `
  --runtime-image "$jlinkDir" `
  --win-console `
  --java-options "-Dfile.encoding=UTF-8" `
  --java-options "-Dspring.config.additional-location=optional:file:./,optional:file:./config/"
if ($LASTEXITCODE -ne 0) { throw "jpackage failed" }

Write-Host "=== Step 4: Copy external config ==="
Copy-Item -Force src/main/resources/application.yml "$appDir/"
Copy-Item -Force src/main/resources/application.properties "$appDir/"

Write-Host "=== Step 5: Generate launcher scripts ==="
@"
@echo off
"%~dp0$AppName.exe" %*
"@ | Out-File -Encoding ASCII "$appDir\start.bat"

Write-Host "=== Step 6: Create archive ==="
$archiveName = "${AppName}-windows-x64.zip"
Push-Location $distDir
Compress-Archive -Path $AppName -DestinationPath $archiveName
Write-Host "  $archiveName"
Pop-Location

Write-Host "=== Done: $appDir ==="
Get-ChildItem "$appDir\start.bat"
Get-ChildItem "$appDir\$AppName.exe" -ErrorAction SilentlyContinue
Get-ChildItem "$distDir\$archiveName"
