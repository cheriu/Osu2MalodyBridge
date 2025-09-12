package net.omastore.malodystore.util

import com.github.benmanes.caffeine.cache.Caffeine
import net.omastore.malodystore.model.ChartStoreListQueryParameters
import net.omastore.malodystore.model.ChartStorePromoteQueryParameters
import org.springframework.stereotype.Component
import java.util.concurrent.TimeUnit

@Component
final class CursorCacheFactory {
    inline fun <reified K : Any> createCursorCache(
        maxSize: Long = 10_000,
        expireAfterMinutes: Long = 60,
    ) = Caffeine
        .newBuilder()
        .maximumSize(maxSize)
        .expireAfterWrite(expireAfterMinutes, TimeUnit.MINUTES)
        .recordStats()
        .build<K, String>()
}

@Component
class ChartStoreListCursorCache(
    private val factory: CursorCacheFactory,
) {
    private val cache = factory.createCursorCache<ChartStoreListQueryParameters>()

    fun getCursor(key: ChartStoreListQueryParameters): String = cache.getIfPresent(key) ?: ""

    fun putCursor(
        key: ChartStoreListQueryParameters,
        cursor: String,
    ) {
        cache.put(key, cursor)
    }
}

@Component
class ChartStorePromoteCursorCache(
    private val factory: CursorCacheFactory,
) {
    private val cache = factory.createCursorCache<ChartStorePromoteQueryParameters>()

    fun getCursor(key: ChartStorePromoteQueryParameters): String = cache.getIfPresent(key) ?: ""

    fun putCursor(
        key: ChartStorePromoteQueryParameters,
        cursor: String,
    ) {
        cache.put(key, cursor)
    }
}
