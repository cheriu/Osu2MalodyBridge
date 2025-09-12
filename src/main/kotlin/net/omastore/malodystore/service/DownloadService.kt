package net.omastore.malodystore.service

import java.io.File

interface DownloadService {
    suspend fun download(
        url: String,
        targetDir: File,
        fileName: String,
    ): Boolean
}
