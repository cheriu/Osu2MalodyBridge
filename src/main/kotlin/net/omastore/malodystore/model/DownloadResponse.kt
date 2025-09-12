package net.omastore.malodystore.model

data class DownloadResponse(
    val code: Int,
    val items: List<DownloadItem>,
    val sid: Int,
    val cid: Int,
)
