package net.omastore.malodystore.service

import net.omastore.malodystore.util.BeatmapId2BeatmapCache
import net.omastore.malodystore.util.osuApiV2.Beatmap
import net.omastore.malodystore.util.osuApiV2.OsuApiV2

fun getBeatmapFromCacheOrRemote(
    beatmapId: Int,
    osuApiV2: OsuApiV2,
    cache: BeatmapId2BeatmapCache,
): Beatmap? =
    cache.getBeatmap(beatmapId)
        ?: osuApiV2
            .getBeatmap(beatmapId)
            ?.also { beatmap -> cache.putBeatmap(beatmapId, beatmap) }
