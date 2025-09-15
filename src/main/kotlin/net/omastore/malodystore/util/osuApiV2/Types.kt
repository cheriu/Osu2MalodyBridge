package net.omastore.malodystore.util.osuApiV2

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/**
 * https://osu.ppy.sh/docs/index.html#beatmapset-covers
 */
@Serializable
data class Covers(
    val cover: String,
    @SerialName("cover@2x")
    val cover2x: String,
    val card: String,
    @SerialName("card@2x")
    val card2x: String,
    val list: String,
    @SerialName("list@2x")
    val list2x: String,
    val slimcover: String,
    @SerialName("slimcover@2x")
    val slimcover2x: String,
)

@Serializable
data class NominationsSummary(
    val current: Int,
    val eligible_main_rulesets: List<String>,
    val required_meta: RequiredMeta,
)

@Serializable
data class RequiredMeta(
    val main_ruleset: Int,
    val non_main_ruleset: Int,
)

@Serializable
data class Availability(
    val download_disabled: Boolean,
    val more_information: String? = null,
)

@Serializable
data class Beatmap(
    val beatmapset_id: Int,
    val difficulty_rating: Double,
    val id: Int,
    val mode: String,
    val status: String,
    val total_length: Int,
    val user_id: Int,
    val version: String,
    val accuracy: Double,
    val ar: Float,
    val bpm: Float,
    val convert: Boolean,
    val count_circles: Int,
    val count_sliders: Int,
    val count_spinners: Int,
    val cs: Float,
    val deleted_at: String? = null,
    val drain: Double,
    val hit_length: Int,
    val is_scoreable: Boolean,
    val last_updated: String,
    val mode_int: Int,
    val passcount: Int,
    val playcount: Int,
    val ranked: Int,
    val url: String,
    val checksum: String,
    val max_combo: Int,
)

/**
 * https://osu.ppy.sh/docs/index.html#beatmapset
 */
@Serializable
data class Beatmapset(
    val artist: String,
    @SerialName("artist_unicode")
    val artistUnicode: String,
    val covers: Covers,
    val creator: String,
    val favourite_count: Int,
    val id: Int,
    val nsfw: Boolean,
    val offset: Int,
    val play_count: Int,
    @SerialName("preview_url")
    val previewUrl: String,
    val source: String? = null,
    val spotlight: Boolean,
    val status: String,
    val title: String,
    @SerialName("title_unicode")
    val titleUnicode: String,
    val user_id: Int,
    val video: Boolean,
    val bpm: Float,
    val can_be_hyped: Boolean,
    val deleted_at: String? = null,
    val discussion_enabled: Boolean,
    val discussion_locked: Boolean,
    val is_scoreable: Boolean,
    val last_updated: String,
    val legacy_thread_url: String? = null,
    val nominations_summary: NominationsSummary,
    val ranked: Int,
    val ranked_date: String,
    val rating: Float,
    val storyboard: Boolean,
    val submitted_date: String,
    val tags: String,
    val availability: Availability,
    val has_favourited: Boolean? = false,
    val beatmaps: List<Beatmap>,
    val pack_tags: List<String>? = null,
)

@Serializable
data class BeatmapsetsSearchResponse(
    val beatmapsets: List<Beatmapset>,
    val total: Int,
    val cursor_string: String?, // for pagination
)

/**
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
 */
@Serializable
data class BeatmapsetsSearchParameters(
    val general: General? = null,
    val mode: Mode = Mode.ANY,
    val category: Categories = Categories.HAS_LEADERBOARD,
    val explicitContent: ExplicitContent = ExplicitContent.HIDE,
    val genre: Genre = Genre.ANY,
    val language: Language = Language.ANY,
    val extra: Extra? = null,
    val rankAchieved: RankAchieved? = null,
    val played: Played = Played.ANY,
    val sort: SortBy = SortBy.RANKED_DESC,
    val keywords: String = "",
) {
    enum class General(
        override val value: String,
    ) : SearchOptions<String> {
        RECOMMENDED_DIFFICULTY("recommended"),
        INCLUDE_CONVERTED_BEATMAPS("converts"),
        SUBSCRIBED_MAPPERS("follows"),
        SPOTLIGHTED_BEATMAPS("spotlights"),
        FEATURED_ARTISTS("featured_artists"),
    }

    enum class Mode(
        override val value: Int?,
    ) : SearchOptions<Int?> {
        ANY(null),
        OSU(0),
        OSU_TAIKO(1),
        OSU_CATCH(2),
        OSU_MANIA(3),
    }

    enum class Categories(
        override val value: String? = null,
    ) : SearchOptions<String?> {
        ANY("any"),
        HAS_LEADERBOARD(null),
        RANKED("ranked"),
        QUALIFIED("qualified"),
        LOVED("loved"),
        FAVOURITES("favourites"),
        PENDING("pending"),
        WIP("wip"),
        GRAVEYARD("graveyard"),
        MY_MAPS("mine"),
    }

    enum class ExplicitContent(
        override val value: Boolean?,
    ) : SearchOptions<Boolean?> {
        HIDE(null),
        SHOW(true),
    }

    enum class Genre(
        override val value: Int?,
    ) : SearchOptions<Int?> {
        ANY(null),
        UNSPECIFIED(1),
        VIDEO_GAME(2),
        ANIME(3),
        ROCK(4),
        POP(5),
        OTHER(6),
        NOVELTY(7),
        HIP_HOP(9),
        ELECTRONIC(10),
        METAL(11),
        CLASSICAL(12),
        FOLK(13),
        JAZZ(14),
    }

    enum class Language(
        override val value: Int?,
    ) : SearchOptions<Int?> {
        ANY(null),
        ENGLISH(2),
        CHINESE(4),
        FRENCH(7),
        GERMAN(8),
        ITALIAN(11),
        JAPANESE(3),
        KOREAN(6),
        SPANISH(10),
        SWEDISH(9),
        RUSSIAN(12),
        POLISH(13),
        INSTRUMENTAL(5),
        UNSPECIFIED(1),
        OTHER(14),
    }

    enum class Extra(
        override val value: String,
    ) : SearchOptions<String> {
        HAS_VIDEO("video"),
        HAS_STORYBOARD("storyboard"),
    }

    enum class RankAchieved(
        override val value: String,
    ) : SearchOptions<String> {
        SILVER_SS("XH"), // Silver SS
        SS("X"), // SS
        SILVER_S("SH"), // Silver S
        S("S"), // S
        A("A"),
        B("B"),
        C("C"),
        D("D"),
    }

    enum class Played(
        override val value: String?,
    ) : SearchOptions<String?> {
        ANY(null),
        PLAYED("played"),
        UNPLAYED("unplayed"),
    }

    enum class SortBy(
        override val value: String?,
    ) : SearchOptions<String?> {
        TITLE_DESC("title_desc"),
        TITLE_ASC("title_asc"),
        ARTIST_DESC("artist_desc"),
        ARTIST_ASC("artist_asc"),
        DIFFICULTY_DESC("difficulty_desc"),
        DIFFICULTY_ASC("difficulty_asc"),
        RANKED_DESC(null),
        RANKED_ASC("ranked_asc"),
        RATING_DESC("rating_desc"),
        RATING_ASC("rating_asc"),
        PLAYS_DESC("plays_desc"),
        PLAYS_ASC("plays_asc"),
        FAVOURITES_DESC("favourites_desc"),
        FAVOURITES_ASC("favourites_asc"),
    }
}

interface SearchOptions<T> {
    val value: T
}

fun <T, V> (T?).toStringOrEmpty(): String where T : Enum<T>, T : SearchOptions<V> = this?.value?.toString() ?: ""
