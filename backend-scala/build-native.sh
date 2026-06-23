#!/bin/bash
set -e

echo "=== Building Scala Native Image ==="

# Build the fat JAR
echo "Building fat JAR..."
sbt assembly

# Find the JAR
JAR_PATH=$(find target/scala-3.3.7 -name "investment-portfolio-backend-*.jar" | head -1)
if [ -z "$JAR_PATH" ]; then
    echo "ERROR: Could not find fat JAR"
    exit 1
fi

echo "Found JAR: $JAR_PATH"

# Extract SQLite native-image feature class from the fat JAR
mkdir -p /tmp/sqlite-feature/org/sqlite/nativeimage
unzip -p "$JAR_PATH" META-INF/versions/9/org/sqlite/nativeimage/SqliteJdbcFeature.class > /tmp/sqlite-feature/org/sqlite/nativeimage/SqliteJdbcFeature.class
unzip -p "$JAR_PATH" META-INF/versions/9/org/sqlite/nativeimage/SqliteJdbcFeature\$SqliteJdbcFeatureException.class > /tmp/sqlite-feature/org/sqlite/nativeimage/SqliteJdbcFeature\$SqliteJdbcFeatureException.class

# Run native-image from Docker (GraalVM native-image only works on Linux)
echo "Running GraalVM native-image from Docker container..."
docker run --rm \
    -v "$PWD":/app \
    -v /tmp/sqlite-feature:/sqlite-feature \
    -w /app \
    --memory=16g \
    --cpus=4 \
    bellsoft/liberica-native-image-kit-container \
    native-image \
    -cp /sqlite-feature \
    -jar "$JAR_PATH" \
    --no-fallback \
    --initialize-at-build-time=org.slf4j,org.sqlite.util.ProcessRunner \
    --enable-url-protocols=http,https \
    --features=org.sqlite.nativeimage.SqliteJdbcFeature \
    -H:+ReportExceptionStackTraces \
    -H:+AddAllCharsets \
    -H:IncludeResourceBundles=org.sqlite.locale \
    -H:DeadlockWatchdogInterval=300 \
    -o /app/target/scala-3.3.7/investment-portfolio-backend

echo "=== Native image built successfully at target/scala-3.3.7/investment-portfolio-backend ==="
