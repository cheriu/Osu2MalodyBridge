package net.omastore.malodystore.util

import com.github.benmanes.caffeine.cache.Caffeine
import net.omastore.malodystore.util.osuApiV2.Beatmap
import org.springframework.stereotype.Component
import java.util.concurrent.TimeUnit

@Component
class BeatmapId2BeatmapCache {
    private val cache =
        Caffeine
            .newBuilder()
            .maximumSize(1000)
            .expireAfterWrite(30, TimeUnit.MINUTES)
            .build<Int, Beatmap>()

    fun getBeatmap(key: Int): Beatmap? = cache.getIfPresent(key)

    fun putBeatmap(
        key: Int,
        beatmap: Beatmap,
    ) {
        cache.put(key, beatmap)
    }
}
