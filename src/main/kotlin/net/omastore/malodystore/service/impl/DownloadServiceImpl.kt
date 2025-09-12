package net.omastore.malodystore.service.impl

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import net.omastore.malodystore.service.DownloadService
import okhttp3.OkHttpClient
import okhttp3.Request
import org.springframework.stereotype.Service
import java.io.File
import java.io.IOException

@Service
class DownloadServiceImpl : DownloadService {
    override suspend fun download(
        url: String,
        targetDir: File,
        fileName: String,
    ): Boolean =
        withContext(Dispatchers.IO) {
            val client = OkHttpClient()

            val request =
                Request
                    .Builder()
                    .url(url)
                    .build()

            try {
                val response = client.newCall(request).execute()

                if (!response.isSuccessful) throw IOException("Unexpected code $response")
                val body = response.body

                val targetFile = File(targetDir, fileName)

                targetFile.parentFile?.let {
                    if (!it.exists()) {
                        it.mkdirs()
                    }
                }

                body.byteStream().use { inputStream ->
                    targetFile.outputStream().use { outputStream ->
                        val buffer = ByteArray(8192)
                        var bytesRead: Int
                        while (inputStream.read(buffer).also { bytesRead = it } != -1) {
                            outputStream.write(buffer, 0, bytesRead)
                        }
                    }
                }
                true
            } catch (e: Exception) {
                e.printStackTrace()
                false
            }
        }
}
