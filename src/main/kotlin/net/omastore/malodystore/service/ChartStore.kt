package net.omastore.malodystore.service

import jakarta.servlet.http.HttpServletRequest
import jakarta.servlet.http.HttpServletResponse
import net.omastore.malodystore.model.Chart
import net.omastore.malodystore.model.ChartStoreListQueryParameters
import net.omastore.malodystore.model.ChartStorePromoteQueryParameters
import net.omastore.malodystore.model.DownloadResponse
import net.omastore.malodystore.model.PagedResponse
import net.omastore.malodystore.model.Song

interface ChartStore {
    /**
     * Purpose: Get a list of charts under the specified query conditions
     */
    fun list(parameters: ChartStoreListQueryParameters): PagedResponse<Song>

    fun promote(parameters: ChartStorePromoteQueryParameters): PagedResponse<Song>

    fun charts(
        sid: Int,
        beta: Int = 0,
        mode: Int = -1,
        from: Int = 0,
        promote: Int = 0,
    ): PagedResponse<Chart>

    fun download(
        cid: Int,
        request: HttpServletRequest,
    ): DownloadResponse

    fun sendChartResource(
        cid: Int,
        type: String,
        response: HttpServletResponse,
    )
}
