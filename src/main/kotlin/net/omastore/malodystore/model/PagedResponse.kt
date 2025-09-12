package net.omastore.malodystore.model

data class PagedResponse<T>(
    val data: List<T>,
    val code: Int = 0,
    val hasMore: Boolean,
    val next: Int,
)
