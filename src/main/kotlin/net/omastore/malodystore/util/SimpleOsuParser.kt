package net.omastore.malodystore.util

import java.io.InputStream

fun parseForBackground(inputStream: InputStream): String? {
    var currentSection: String? = null
    var backgroundFilename: String? = null

    inputStream.bufferedReader().useLines { lines ->
        for (line in lines) {
            val trimmedLine = line.trim()
            if (trimmedLine.isEmpty() || trimmedLine.startsWith("//")) {
                continue
            }

            if (trimmedLine.startsWith("[") && trimmedLine.endsWith("]")) {
                currentSection = trimmedLine.substring(1, trimmedLine.length - 1)
                continue
            }

            when (currentSection) {
                "Events" -> {
                    val parts = trimmedLine.split(',').map { it.trim() }
                    if (parts.size >= 3 && parts[0] == "0" && parts[1] == "0") {
                        backgroundFilename = parts[2].removeSurrounding("\"", "\"")
                        break
                    }
                }
            }
        }
    }
    return backgroundFilename
}

fun parseForAudioFileName(inputStream: InputStream): String? {
    var currentSection: String? = null
    var audioFilename: String? = null

    inputStream.bufferedReader().useLines { lines ->
        for (line in lines) {
            val trimmedLine = line.trim()
            if (trimmedLine.isEmpty() || trimmedLine.startsWith("//")) {
                continue
            }

            if (trimmedLine.startsWith("[") && trimmedLine.endsWith("]")) {
                currentSection = trimmedLine.substring(1, trimmedLine.length - 1)
                continue
            }

            when (currentSection) {
                "General" -> {
                    val parts = trimmedLine.split(':', limit = 2)
                    if (parts.size >= 2) {
                        val key = parts[0].trim()
                        if (key == "AudioFilename") {
                            audioFilename = parts[1].trim().removeSurrounding("\"", "\"")
                        }
                    }
                }
            }
        }
    }
    return audioFilename
}

fun parseForAudioFileNameBackground(inputStream: InputStream): Pair<String?, String?> {
    var currentSection: String? = null
    var audioFilename: String? = null
    var backgroundFilename: String? = null

    inputStream.bufferedReader().useLines { lines ->
        for (line in lines) {
            val trimmedLine = line.trim()
            if (trimmedLine.isEmpty() || trimmedLine.startsWith("//")) {
                continue
            }

            if (trimmedLine.startsWith("[") && trimmedLine.endsWith("]")) {
                currentSection = trimmedLine.substring(1, trimmedLine.length - 1)
                continue
            }

            when (currentSection) {
                "General" -> {
                    val parts = trimmedLine.split(':', limit = 2)
                    if (parts.size >= 2) {
                        val key = parts[0].trim()
                        if (key == "AudioFilename") {
                            audioFilename = parts[1].trim().removeSurrounding("\"", "\"")
                        }
                    }
                }
                "Events" -> {
                    val parts = trimmedLine.split(',').map { it.trim() }
                    if (parts.size >= 3 && parts[0] == "0" && parts[1] == "0") {
                        backgroundFilename = parts[2].removeSurrounding("\"", "\"")
                        break
                    }
                }
            }
        }
    }
    return Pair(audioFilename, backgroundFilename)
}
