package net.omastore.malodystore.util.osuApiV2

interface OsuApiV2 {
    /**
     * 搜索谱面
     *
     * @param cursorString for pagination
     */
    fun beatmapsetsSearch(
        beatmapsetsSearchParameters: BeatmapsetsSearchParameters,
        cursorString: String = "",
    ): BeatmapsetsSearchResponse?

    fun getBeatmapset(beatmapsetId: Int): Beatmapset?

    fun getBeatmap(beatmapId: Int): Beatmap?
}
