package net.omastore.malodystore.service.impl

import jakarta.servlet.http.HttpServletRequest
import jakarta.servlet.http.HttpServletResponse
import kotlinx.coroutines.runBlocking
import net.omastore.malodystore.config.OsuAuthSecret
import net.omastore.malodystore.controller.API_BASE_PATH
import net.omastore.malodystore.model.Chart
import net.omastore.malodystore.model.ChartStoreListQueryParameters
import net.omastore.malodystore.model.ChartStorePromoteQueryParameters
import net.omastore.malodystore.model.DownloadItem
import net.omastore.malodystore.model.DownloadResponse
import net.omastore.malodystore.model.PagedResponse
import net.omastore.malodystore.model.Song
import net.omastore.malodystore.model.toBeatmapsetsSearchParameters
import net.omastore.malodystore.model.toNextListQueryParameters
import net.omastore.malodystore.service.ChartStore
import net.omastore.malodystore.service.DownloadService
import net.omastore.malodystore.util.ChartStoreListCursorCache
import net.omastore.malodystore.util.ChartStorePromoteCursorCache
import net.omastore.malodystore.util.osuApiV2.Beatmap
import net.omastore.malodystore.util.osuApiV2.Beatmapset
import net.omastore.malodystore.util.osuApiV2.BeatmapsetsSearchResponse
import net.omastore.malodystore.util.osuApiV2.impl.OsuApiV2Impl
import net.omastore.malodystore.util.parseForAudioFileName
import net.omastore.malodystore.util.parseForAudioFileNameBackground
import net.omastore.malodystore.util.parseForBackground
import org.apache.commons.compress.archivers.zip.ZipArchiveEntry
import org.apache.commons.compress.archivers.zip.ZipArchiveInputStream
import org.apache.commons.compress.archivers.zip.ZipFile
import org.slf4j.LoggerFactory
import org.springframework.beans.factory.annotation.Autowired
import org.springframework.beans.factory.annotation.Value
import org.springframework.stereotype.Service
import java.io.File
import java.io.FileInputStream
import java.io.InputStream
import java.security.MessageDigest
import java.time.Instant
import kotlin.collections.emptyList
import kotlin.math.floor

@Service
class ChartStoreImpl(
    osuAuthSecret: OsuAuthSecret,
    private val chartStoreListCursorCache: ChartStoreListCursorCache,
    private val chartStorePromoteCursorCache: ChartStorePromoteCursorCache,
) : ChartStore {
    private val logger = LoggerFactory.getLogger(javaClass)
    private val osuApiV2 = OsuApiV2Impl(osuAuthSecret.clientId, osuAuthSecret.clientSecret)

    @Autowired
    private lateinit var downloadService: DownloadService

    @Value("\${malody.server.url}")
    private var serverUrl: String = ""

    @Value("\${malody.server.tmp}")
    private var tmpDir: String = "/tmp"

    override fun list(parameters: ChartStoreListQueryParameters): PagedResponse<Song> {
        if (parameters.from == 0) { // first time query for this parameters
            val beatmapSetsSearchResponse =
                osuApiV2
                    .beatmapsetsSearch(
                        parameters.toBeatmapsetsSearchParameters(),
                    )?.also { response ->
                        chartStoreListCursorCache.putCursor(parameters.toNextListQueryParameters(response), response.cursor_string ?: "")
                    }
            return beatmapSetsSearchResponse?.toPagedResponseSong(from = parameters.from)
                ?: PagedResponse<Song>(hasMore = false, next = 0, data = emptyList(), code = -1).also {
                    logger.error("List: Error while getting beatmapsets search")
                }
        } else {
            val beatmapsetsSearchResponse =
                osuApiV2
                    .beatmapsetsSearch(
                        parameters.toBeatmapsetsSearchParameters(),
                        cursorString = chartStoreListCursorCache.getCursor(parameters),
                    )?.also { response ->
                        chartStoreListCursorCache.putCursor(parameters.toNextListQueryParameters(response), response.cursor_string ?: "")
                    }
            return beatmapsetsSearchResponse?.toPagedResponseSong(from = parameters.from)
                ?: PagedResponse<Song>(hasMore = false, next = 0, data = emptyList(), code = -1).also {
                    logger.error("List: Error while getting beatmapsets search")
                }
        }
    }

    override fun promote(parameters: ChartStorePromoteQueryParameters): PagedResponse<Song> {
        if (parameters.from == 0) { // first time query for this parameters
            val beatmapSetsSearchResponse =
                osuApiV2
                    .beatmapsetsSearch(
                        parameters.toBeatmapsetsSearchParameters(),
                    )?.also { response ->
                        chartStorePromoteCursorCache.putCursor(parameters.toNextListQueryParameters(response), response.cursor_string ?: "")
                    }
            return beatmapSetsSearchResponse?.toPagedResponseSong(from = parameters.from)
                ?: PagedResponse<Song>(hasMore = false, next = 0, data = emptyList(), code = -1).also {
                    logger.error("List: Error while getting beatmapsets search")
                }
        } else {
            val beatmapsetsSearchResponse =
                osuApiV2
                    .beatmapsetsSearch(
                        parameters.toBeatmapsetsSearchParameters(),
                        cursorString = chartStorePromoteCursorCache.getCursor(parameters),
                    )?.also { response ->
                        chartStorePromoteCursorCache.putCursor(parameters.toNextListQueryParameters(response), response.cursor_string ?: "")
                    }
            return beatmapsetsSearchResponse?.toPagedResponseSong(from = parameters.from)
                ?: PagedResponse<Song>(hasMore = false, next = 0, data = emptyList(), code = -1).also {
                    logger.error("List: Error while getting beatmapsets search")
                }
        }
    }

    private fun Beatmapset.toPagedResponseChart(): PagedResponse<Chart> =
        PagedResponse<Chart>(
            hasMore = false,
            next = 0,
            data =
                this.beatmaps.map {
                    it.toMalodyChart()
                },
            code = -1,
        )

    private fun BeatmapsetsSearchResponse.toPagedResponseSong(from: Int): PagedResponse<Song> {
        return if (this.total == 0) { // no available song
            PagedResponse<Song>(hasMore = false, next = 0, data = emptyList(), code = 1)
        } else {
            return if (this.cursor_string == null) { // no more song
                PagedResponse<Song>(
                    hasMore = false,
                    next = from,
                    data =
                        this.beatmapsets.map {
                            it.toMalodySong()
                        },
                    code = 0,
                )
            } else {
                PagedResponse<Song>(
                    code = 0,
                    hasMore = true,
                    next = from + this.beatmapsets.size,
                    data =
                        this.beatmapsets.map {
                            it.toMalodySong()
                        },
                )
            }
        }
    }

    override fun charts(
        sid: Int,
        beta: Int,
        mode: Int,
        from: Int,
        promote: Int,
    ): PagedResponse<Chart> {
        val beatmapset = osuApiV2.getBeatmapset(beatmapsetId = sid)
        return beatmapset?.toPagedResponseChart()
            ?: PagedResponse<Chart>(hasMore = false, next = 0, data = emptyList(), code = -1)
                .also {
                    logger.error("No beatmapset found")
                }
    }

    override fun download(
        cid: Int,
        request: HttpServletRequest,
    ): DownloadResponse {
        val beatmap =
            osuApiV2.getBeatmap(beatmapId = cid) ?: return DownloadResponse(
                cid = cid,
                items = emptyList(),
                code = -2,
                sid = 0,
            )
        val beatmapsetId = beatmap.beatmapset_id
        val oszFile = File(tmpDir, "${beatmapsetId}n.osz")

        val host = request.getHeader("Host")
        val baseUrl = "http://$host/$API_BASE_PATH"

        if (!oszFile.exists()) {
            val isSuccess =
                runBlocking {
                    downloadService.download(
                        url = "https://catboy.best/d/${beatmap.beatmapset_id}n",
                        targetDir = File(tmpDir),
                        fileName = "${beatmap.beatmapset_id}n.osz",
                    )
                }
            if (!isSuccess) {
                return DownloadResponse(
                    cid = cid,
                    items = emptyList(),
                    code = -2,
                    sid = beatmap.beatmapset_id,
                )
            }
        }
        val items =
            buildDownloadItems(beatmap, oszFile, baseUrl) ?: return DownloadResponse(
                cid = cid,
                items = emptyList(),
                code = -2,
                sid = beatmap.beatmapset_id,
            )
        return DownloadResponse(
            cid = cid,
            code = 0,
            sid = beatmapsetId,
            items = items,
        )
    }

    private fun buildDownloadItems(
        beatmap: Beatmap,
        zipFile: File,
        baseUrl: String,
    ): List<DownloadItem>? {
        val zip =
            ZipFile
                .builder()
                .setFile(zipFile)
                .setUseUnicodeExtraFields(true)
                .get()

        val chartInputStream = getDecompressedChart(zip, beatmap)
        if (chartInputStream != null) {
            val audioFileNameAndBackground = parseForAudioFileNameBackground(chartInputStream)
            val audio = audioFileNameAndBackground.first
            val background = audioFileNameAndBackground.second

            if (audio == null || background == null) {
                return null
            }

            return listOf(
                DownloadItem(
                    name = "${beatmap.version} Lv.${floor(beatmap.difficulty_rating + 0.5)}.osu",
                    hash = beatmap.checksum,
                    file = "$baseUrl/${beatmap.id}?type=chart",
                ),
                DownloadItem(
                    name = audio,
                    hash = getZipEntryMD5(zipFile, audio) ?: "",
                    file = "$baseUrl/${beatmap.id}?type=audio",
                ),
                DownloadItem(
                    name = background,
                    hash = getZipEntryMD5(zipFile, background) ?: "",
                    file = "$baseUrl/${beatmap.id}?type=bg",
                ),
            )
        } else {
            return null
        }
    }

    override fun sendChartResource(
        cid: Int,
        type: String,
        response: HttpServletResponse,
    ) {
        osuApiV2.getBeatmap(beatmapId = cid)?.let { beatmap ->
            val oszFilePath = "$tmpDir/${beatmap.beatmapset_id}n.osz"
            val oszFile = File(oszFilePath)

            if (!oszFile.exists()) {
                response.sendError(HttpServletResponse.SC_NOT_FOUND, "osz file not found")
                logger.error("osz file not found, cid = $cid")
                return
            }

            try {
                val zipArchive =
                    ZipFile
                        .builder()
                        .setFile(oszFile)
                        .setUseUnicodeExtraFields(true)
                        .get()
                zipArchive.use { zip ->
                    val entries = zip.entries
                    when (type) {
                        "chart" -> {
                            val chartInputStream = getDecompressedChart(zip, beatmap)
                            if (chartInputStream == null) {
                                response.sendError(HttpServletResponse.SC_NOT_FOUND, "Chart file not found")
                                return
                            }

                            chartInputStream.copyTo(response.outputStream)
                            response.outputStream.flush()
                        }
                        "audio" -> {
                            val chartInputStream = getDecompressedChart(zip, beatmap)
                            if (chartInputStream == null) {
                                response.sendError(HttpServletResponse.SC_NOT_FOUND, "Chart file not found")
                                return
                            }
                            val audioFileName = parseForAudioFileName(chartInputStream)
                            val audio =
                                entries
                                    .asSequence()
                                    .filter { it.name == audioFileName }
                                    .toList()
                            when (audio.size) {
                                0 -> {
                                    response
                                        .sendError(
                                            HttpServletResponse.SC_NOT_FOUND,
                                            "audio file not found",
                                        ).also { logger.error("audio file not found") }
                                }
                                1 -> {
                                    zipArchive.getInputStream(audio[0]).use { input ->
                                        val output = response.outputStream
                                        input.copyTo(output)
                                        output.flush()
                                    }
                                }
                                else ->
                                    response.sendError(HttpServletResponse.SC_CONFLICT, "multiple audio file candidates").also {
                                        logger.error("multiple audio file candidates")
                                    }
                            }
                        }
                        "bg" -> {
                            val chartInputStream = getDecompressedChart(zip, beatmap)
                            if (chartInputStream == null) {
                                response.sendError(HttpServletResponse.SC_NOT_FOUND, "Chart file not found")
                                return
                            }
                            val background = parseForBackground(chartInputStream)
                            val bg =
                                entries
                                    .asSequence()
                                    .filter { it.name == background }
                                    .toList()
                            when (bg.size) {
                                0 -> {
                                    response.sendError(HttpServletResponse.SC_NOT_FOUND, "bg file not found")
                                }
                                1 -> {
                                    zipArchive.getInputStream(bg[0]).use { input ->
                                        val output = response.outputStream
                                        input.copyTo(output)
                                        output.flush()
                                    }
                                }
                                else ->
                                    response.sendError(HttpServletResponse.SC_CONFLICT, "multiple bg file candidates").also {
                                        logger.error("multiple bg file candidates")
                                    }
                            }
                        }
                    }
                }
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }
}

private fun String.basename(): String = this.substringBeforeLast(".", this)

private fun String.basenameLower() = basename().lowercase()

private fun getDecompressedChart(
    zip: ZipFile,
    beatmap: Beatmap,
): InputStream? {
    val entries = zip.entries

    val chartEntries =
        entries
            .asSequence()
            .filter { it.name.endsWith(".osu", ignoreCase = true) }
            .toList()
    when (chartEntries.size) {
        0 -> {
            return null
        }
        else -> {
            var matchedEntry: ZipArchiveEntry? = null
            for (entry in chartEntries) {
                zip.getInputStream(entry).use { input ->
                    val calculatedMd5 = calculateMD5(input)

                    if (calculatedMd5 == beatmap.checksum) {
                        matchedEntry = entry
                        break
                    }
                }
            }
            return matchedEntry?.let { zip.getInputStream(it) }
        }
    }
}

private fun getZipEntryMD5(
    zipFile: File,
    targetFilename: String,
): String? {
    val targetBase = targetFilename.basenameLower()
    FileInputStream(zipFile).use { fileInputStream ->
        ZipArchiveInputStream(fileInputStream, "UTF-8", false, true).use { zis ->
            var entry: ZipArchiveEntry? = zis.nextEntry
            while (entry != null) {
                val entryName = entry.name
                val entryBase = entryName.basenameLower()
                if (entryBase == targetBase && !entry.isDirectory) {
                    return calculateMD5(zis)
                }
                entry = zis.nextEntry
            }
        }
    }
    return null // 未找到匹配的文件
}

private fun calculateMD5(inputStream: InputStream): String {
    val digest = MessageDigest.getInstance("MD5")
    val buffer = ByteArray(8192)
    var bytesRead: Int
    while (inputStream.read(buffer).also { bytesRead = it } != -1) {
        digest.update(buffer, 0, bytesRead)
    }
    val hashBytes = digest.digest()
    return hashBytes.joinToString(separator = "") { "%02x".format(it) }
}

private fun Beatmapset.toMalodySong(): Song =
    Song(
        sid = this.id,
        cover = this.covers.list2x,
        length = this.getLength(),
        bpm = this.bpm,
        title = this.title,
        artist = this.artist,
        mode = 0,
        time = Instant.parse(this.last_updated).epochSecond,
    )

private fun Beatmapset.getLength(): Int = beatmaps.minBy { it.total_length }.total_length

private fun Beatmap.toMalodyChart(): Chart =
    Chart(
        cid = this.id,
        uid = this.user_id,
        creator = this.user_id.toString(), // TODO()
        version = "${this.version} Lv.${floor(this.difficulty_rating + 0.5).toInt()}",
        level = floor(this.difficulty_rating + 0.5).toInt(),
        length = this.total_length,
        type = 1,
        size = 0,
        mode = 0,
    )
