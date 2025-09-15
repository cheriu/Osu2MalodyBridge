FROM gradle:8.14.2-jdk17 as build

WORKDIR /app

COPY gradle/ gradle/
COPY gradlew .
COPY build.gradle.kts .
COPY settings.gradle.kts .

COPY src/ src/

RUN ./gradlew clean build -x test


FROM openjdk:17-jdk-slim

WORKDIR /app

COPY --from=build /app/build/libs/app-1.0.0.jar /app/app.jar

EXPOSE 8081

ENTRYPOINT ["java", "-jar", "app.jar"]