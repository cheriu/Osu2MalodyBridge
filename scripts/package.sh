#!/bin/bash
# Build and package MalodyStore as a self-contained app image (Linux).
# Requires JDK 17+ with jlink and jpackage on PATH.
# Output: build/dist/MalodyStore/
set -euo pipefail

APP_NAME="${APP_NAME:-MalodyStore}"
APP_VERSION="${APP_VERSION:-0.0.1}"
JAVA_MODULES="java.base,java.compiler,java.desktop,java.instrument,java.logging,java.management,java.naming,java.net.http,java.security.jgss,java.sql,java.xml,jdk.unsupported,jdk.zipfs"

echo "=== Step 1: Build bootJar ==="
./gradlew bootJar --no-daemon -q

JAR_FILE=$(ls build/libs/*.jar | head -1)
echo "  JAR: $JAR_FILE"

echo "=== Step 2: jlink (stripped JRE) ==="
jlink \
  --output build/jlink-runtime \
  --add-modules "$JAVA_MODULES" \
  --strip-debug --no-header-files --no-man-pages \
  --compress 2

echo "=== Step 3: jpackage (app image) ==="
jpackage \
  --type app-image \
  --name "$APP_NAME" \
  --app-version "$APP_VERSION" \
  --input build/libs \
  --main-jar "$(basename "$JAR_FILE")" \
  --main-class org.springframework.boot.loader.launch.JarLauncher \
  --dest build/dist \
  --runtime-image build/jlink-runtime \
  --java-options "-Dfile.encoding=UTF-8" \
  --java-options "-Dspring.config.additional-location=optional:file:./,optional:file:./config/"

echo "=== Step 4: Copy external config ==="
cp src/main/resources/application.yml "build/dist/$APP_NAME/"
cp src/main/resources/application.properties "build/dist/$APP_NAME/"

echo "=== Step 5: Generate launcher scripts ==="
cat > "build/dist/$APP_NAME/start.sh" << 'SCRIPT'
#!/bin/bash
cd "$(dirname "$0")"
exec bin/MalodyStore "$@"
SCRIPT
chmod +x "build/dist/$APP_NAME/start.sh"

echo "=== Done: build/dist/$APP_NAME/ ==="
ls -lh "build/dist/$APP_NAME/start.sh" "build/dist/$APP_NAME/bin/"
