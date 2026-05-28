# Build and package MalodyStore as a self-contained app image (Windows).
# Requires JDK 17+ with jlink and jpackage on PATH.
# Output: build/dist/MalodyStore/
param(
    [string]$AppName = "MalodyStore",
    [string]$AppVersion = "0.0.1"
)

$ErrorActionPreference = "Stop"
$javaModules = "java.base,java.compiler,java.desktop,java.instrument,java.logging,java.management,java.naming,java.net.http,java.security.jgss,java.sql,java.xml,jdk.unsupported,jdk.zipfs"

Write-Host "=== Step 1: Build bootJar ==="
./gradlew bootJar --no-daemon -q

$jarFile = Get-ChildItem build/libs/*.jar | Select-Object -First 1
Write-Host "  JAR: $($jarFile.Name)"

Write-Host "=== Step 2: jlink (stripped JRE) ==="
jlink `
  --output build/jlink-runtime `
  --add-modules $javaModules `
  --strip-debug --no-header-files --no-man-pages `
  --compress 2

Write-Host "=== Step 3: jpackage (app image) ==="
jpackage `
  --type app-image `
  --name $AppName `
  --app-version $AppVersion `
  --input build/libs `
  --main-jar $jarFile.Name `
  --main-class org.springframework.boot.loader.launch.JarLauncher `
  --dest build/dist `
  --runtime-image build/jlink-runtime `
  --java-options "-Dfile.encoding=UTF-8" `
  --java-options "-Dspring.config.additional-location=optional:file:./,optional:file:./config/"

Write-Host "=== Step 4: Copy external config ==="
Copy-Item -Force src/main/resources/application.yml "build/dist/$AppName/"
Copy-Item -Force src/main/resources/application.properties "build/dist/$AppName/"

Write-Host "=== Step 5: Generate launcher scripts ==="
@"
@echo off
"%~dp0bin\$AppName.exe" %*
"@ | Out-File -Encoding ASCII "build/dist/$AppName/start.bat"

Write-Host "=== Done: build/dist/$AppName/ ==="
Get-ChildItem "build/dist/$AppName/start.bat", "build/dist/$AppName/bin/"
