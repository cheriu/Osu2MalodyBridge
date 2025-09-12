package net.omastore.malodystore.util.osuApiV2.impl

import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.Json
import net.omastore.malodystore.util.osuApiV2.Authentication
import net.omastore.malodystore.util.osuApiV2.Beatmap
import net.omastore.malodystore.util.osuApiV2.Beatmapset
import net.omastore.malodystore.util.osuApiV2.BeatmapsetsSearchParameters
import net.omastore.malodystore.util.osuApiV2.BeatmapsetsSearchResponse
import net.omastore.malodystore.util.osuApiV2.OsuApiV2
import net.omastore.malodystore.util.osuApiV2.toStringOrEmpty
import okhttp3.HttpUrl
import okhttp3.OkHttpClient
import okhttp3.Request
import org.slf4j.LoggerFactory

// @Component
class OsuApiV2Impl(
    clientId: String,
    clientSecret: String,
) : OsuApiV2 {
    val auth = Authentication.getInstance(clientId, clientSecret)

    val client: OkHttpClient = OkHttpClient()

    private val logger = LoggerFactory.getLogger(OsuApiV2Impl::class.java)

    companion object {
        private val json = Json { ignoreUnknownKeys = true }
    }

    override fun beatmapsetsSearch(
        beatmapsetsSearchParameters: BeatmapsetsSearchParameters,
        cursorString: String,
    ): BeatmapsetsSearchResponse? {
        val url =
            HttpUrl
                .Builder()
                .scheme("https")
                .host("osu.ppy.sh")
                .addPathSegments("api/v2/beatmapsets/search")
                .apply {
                    addQueryParameter("e", beatmapsetsSearchParameters.extra.toStringOrEmpty())
                    addQueryParameter("c", beatmapsetsSearchParameters.general.toStringOrEmpty())
                    addQueryParameter("g", beatmapsetsSearchParameters.genre.toStringOrEmpty())
                    addQueryParameter("l", beatmapsetsSearchParameters.language.toStringOrEmpty())
                    addQueryParameter("m", beatmapsetsSearchParameters.mode.toStringOrEmpty())
                    addQueryParameter("nsfw", beatmapsetsSearchParameters.explicitContent.toStringOrEmpty())
                    addQueryParameter("played", beatmapsetsSearchParameters.played.toStringOrEmpty())
                    addQueryParameter("q", beatmapsetsSearchParameters.keywords)
                    addQueryParameter("r", beatmapsetsSearchParameters.rankAchieved.toStringOrEmpty())
                    addQueryParameter("sort", beatmapsetsSearchParameters.sort.toStringOrEmpty())
                    addQueryParameter("s", beatmapsetsSearchParameters.category.toStringOrEmpty())
                    if (cursorString.isNotEmpty()) {
                        addQueryParameter("cursor_string", cursorString)
                    }
                }.build()
                .also { logger.info("osu search url: $it") }

        return try {
            val accessToken =
                runBlocking {
                    auth.getAccessToken()
                }

            val request =
                Request
                    .Builder()
                    .url(url)
                    .get()
                    .header("Authorization", "Bearer $accessToken")
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .build()

            val response = client.newCall(request).execute()
            if (response.isSuccessful) {
                val body = response.body?.string() ?: return null
                logger.trace("Response body: $body")
                return json.decodeFromString<BeatmapsetsSearchResponse>(body)
            } else {
                logger.error("failed to generate beatmapset search response")
                null
            }
        } catch (e: Exception) {
            e.printStackTrace()
            null
        }
    }

    override fun getBeatmapset(beatmapsetId: Int): Beatmapset? {
        val url =
            HttpUrl
                .Builder()
                .scheme("https")
                .host("osu.ppy.sh")
                .addPathSegments("api/v2/beatmapsets/$beatmapsetId")
                .build()
        return try {
            val accessToken =
                runBlocking {
                    auth.getAccessToken()
                }

            val request =
                Request
                    .Builder()
                    .url(url)
                    .get()
                    .header("Authorization", "Bearer $accessToken")
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .build()

            val response = client.newCall(request).execute()
            if (response.isSuccessful) {
                val body = response.body?.string() ?: return null
                return json.decodeFromString<Beatmapset>(body)
            } else {
                logger.error("failed to get beatmapset")
                null
            }
        } catch (e: Exception) {
            e.printStackTrace()
            null
        }
    }

    override fun getBeatmap(beatmapId: Int): Beatmap? {
        val url =
            HttpUrl
                .Builder()
                .scheme("https")
                .host("osu.ppy.sh")
                .addPathSegments("api/v2/beatmaps/$beatmapId")
                .build()
        return try {
            val accessToken =
                runBlocking {
                    auth.getAccessToken()
                }

            val request =
                Request
                    .Builder()
                    .url(url)
                    .get()
                    .header("Authorization", "Bearer $accessToken")
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .build()

            val response = client.newCall(request).execute()
            if (response.isSuccessful) {
                val body = response.body?.string() ?: return null
                return json.decodeFromString<Beatmap>(body)
            } else {
                logger.error("failed to get beatmap")
                null
            }
        } catch (e: Exception) {
            e.printStackTrace()
            null
        }
    }
}
