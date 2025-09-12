package net.omastore.malodystore.controller

import jakarta.servlet.http.HttpServletRequest
import jakarta.servlet.http.HttpServletResponse
import net.omastore.malodystore.model.Chart
import net.omastore.malodystore.model.ChartStoreListQueryParameters
import net.omastore.malodystore.model.ChartStorePromoteQueryParameters
import net.omastore.malodystore.model.DownloadResponse
import net.omastore.malodystore.model.PagedResponse
import net.omastore.malodystore.model.Song
import net.omastore.malodystore.service.BasicInformation
import net.omastore.malodystore.service.ChartStore
import org.slf4j.LoggerFactory
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PathVariable
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.RequestParam
import org.springframework.web.bind.annotation.RestController

const val API_BASE_PATH = "api/store"

@RestController
@RequestMapping("/$API_BASE_PATH")
class StoreController(
    private val basicInformation: BasicInformation,
    private val chartStore: ChartStore,
) {
    private val logger = LoggerFactory.getLogger(javaClass)

    @GetMapping("/info")
    fun serverInfo() = basicInformation.info()

    @GetMapping("/list")
    fun songList(
        @RequestParam(name = "word", defaultValue = "") word: String,
        @RequestParam(name = "org", defaultValue = "0") org: Int,
        @RequestParam(name = "mode", defaultValue = "-1") mode: Int,
        @RequestParam(name = "lvge", defaultValue = "0") lvge: Int,
        @RequestParam(name = "lvle", defaultValue = "0") lvle: Int,
        @RequestParam(name = "beta", defaultValue = "0") beta: Int,
        @RequestParam(name = "from", defaultValue = "0") from: Int,
    ): PagedResponse<Song> {
//        if (org != 0) {
//            TODO("org is not default")
//        }
//        if (mode != -1) {
//            TODO("mode is not default")
//        }
//        if (lvge != 0) {
//            TODO("lvge is not default")
//        }
//        if (lvle != 0) {
//            TODO("lvle is not default")
//        }
//        if (beta != 0) {
//            TODO("beta is not default")
//        }
//        if (from != 0) {
//            TODO("from is not default")
//        }
        return chartStore.list(ChartStoreListQueryParameters(word, org, mode, lvge, lvle, beta, from)).also {
            logger.info("${ChartStoreListQueryParameters(word, org, mode, lvge, lvle, beta, from)}")
        }
    }

    @GetMapping("/promote")
    fun songPromote(
        @RequestParam(name = "org", defaultValue = "0") org: Int,
        @RequestParam(name = "mode", defaultValue = "-1") mode: Int,
        @RequestParam(name = "from", defaultValue = "0") from: Int,
    ): PagedResponse<Song> =
        chartStore.promote(ChartStorePromoteQueryParameters(org, mode, from)).also {
            logger.info("${ChartStorePromoteQueryParameters(org, mode, from)}")
        }

    @GetMapping("/charts")
    fun chartsList(
        @RequestParam(name = "sid") sid: Int,
        @RequestParam(name = "beta", defaultValue = "0") beta: Int,
        @RequestParam(name = "mode", defaultValue = "-1") mode: Int,
        @RequestParam(name = "from", defaultValue = "0") from: Int,
        @RequestParam(name = "promote", defaultValue = "0") promote: Int,
    ): PagedResponse<Chart> =
        chartStore
            .charts(sid = sid)
            .also { logger.info("Charts: {}", it) }

    @GetMapping("/download")
    fun download(
        @RequestParam(name = "cid") cid: Int,
        request: HttpServletRequest,
    ): DownloadResponse = chartStore.download(cid = cid, request = request).also { logger.info("Download: {}", it) }

    private val validTypes = setOf("chart", "audio", "bg")

    @GetMapping("{cid}")
    fun sendChartResource(
        @PathVariable cid: Int,
        @RequestParam(name = "type") type: String,
        response: HttpServletResponse,
    ) {
        if (type !in validTypes) {
            response.sendError(HttpServletResponse.SC_BAD_REQUEST, "Invalid type parameter")
            return
        }
        chartStore.sendChartResource(cid = cid, type = type, response = response)
    }
}
