package net.omastore.malodystore.model

import kotlinx.serialization.Serializable
import net.omastore.malodystore.util.osuApiV2.BeatmapsetsSearchParameters
import net.omastore.malodystore.util.osuApiV2.BeatmapsetsSearchParameters.Mode
import net.omastore.malodystore.util.osuApiV2.BeatmapsetsSearchResponse

/**
 * @param word Search keyword
 * @param org Whether to return the original title
 * @param mode returns the chart of the specified mode, see Mode Definition
 * @param lvge returns the chart whose level is greater than this value
 * @param lvle returns the chart whose level is less than this value
 * @param beta Return to non-stable chart
 * @param from paging start
 */
@Serializable
data class ChartStoreListQueryParameters(
    val word: String = "",
    val org: Int = 0,
    val mode: Int = -1,
    val lvge: Int = 0,
    val lvle: Int = 0,
    val beta: Int = 0,
    val from: Int = 0,
)

enum class MalodyMode(
    val value: Int,
) {
    ANY(-1),
    KEY(0),
    CATCH(3),
    PAD(4),
    TAIKO(5),
    RING(6),
    SLIDE(7),
    LIVE(8),
}

fun ChartStoreListQueryParameters.toBeatmapsetsSearchParameters(): BeatmapsetsSearchParameters =
    BeatmapsetsSearchParameters(
        keywords = this.word,
        mode = Mode.OSU_MANIA, // TODO: support other mode (no plan)
    )

fun ChartStoreListQueryParameters.toNextListQueryParameters(
    beatmapsetsSearchResponse: BeatmapsetsSearchResponse,
): ChartStoreListQueryParameters =
    ChartStoreListQueryParameters(
        word = this.word,
        mode = this.mode,
        lvge = this.lvge,
        lvle = this.lvge,
        beta = this.beta,
        from = this.from + beatmapsetsSearchResponse.beatmapsets.size,
        org = this.org,
    )

/**
 * @param org Whether to return the original title
 * @param mode returns the chart of the specified mode, see Mode Definition
 * @param from paging start
 */
@Serializable
data class ChartStorePromoteQueryParameters(
    val org: Int = 0,
    val mode: Int = -1,
    val from: Int = 0,
)

fun ChartStorePromoteQueryParameters.toBeatmapsetsSearchParameters(): BeatmapsetsSearchParameters =
    BeatmapsetsSearchParameters(
        keywords = "",
        general = BeatmapsetsSearchParameters.General.FEATURED_ARTISTS,
        mode = BeatmapsetsSearchParameters.Mode.OSU_MANIA, // TODO: support other mode (no plan)
    )

fun ChartStorePromoteQueryParameters.toNextListQueryParameters(
    beatmapsetsSearchResponse: BeatmapsetsSearchResponse,
): ChartStorePromoteQueryParameters =
    ChartStorePromoteQueryParameters(
        org = this.org,
        mode = this.mode,
        from = this.from + beatmapsetsSearchResponse.beatmapsets.size,
    )
