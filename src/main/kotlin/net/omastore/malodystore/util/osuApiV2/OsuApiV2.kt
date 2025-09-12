package net.omastore.malodystore.util.osuApiV2

interface OsuApiV2 {
    /**
     * 搜索谱面
     *
     * @param e Extra: {Has Video | Has Storyboard}
     * @param c General: user specific, can be empty or <recommended.>[converts | follows | spotlights | featured_artist]
     * @param g Genre: empty for any
     * @param l Language: empty for any
     * @param m Mode: empty for Any, 3 for osu!mania
     * @param nsfw default empty for false
     * @param q search string
     * @param r Rank Achieved:
     * @param sort
     * @param s Categories: Any Has Leaderboard Ranked Qualified Loved Favourites Pending WIP Graveyard My Maps
     * @param cursorString for pagination
     */
    fun beatmapsetsSearch(
        beatmapsetsSearchParameters: BeatmapsetsSearchParameters,
        cursorString: String = "",
    ): BeatmapsetsSearchResponse?

    fun getBeatmapset(beatmapsetId: Int): Beatmapset?

    fun getBeatmap(beatmapId: Int): Beatmap?
}
