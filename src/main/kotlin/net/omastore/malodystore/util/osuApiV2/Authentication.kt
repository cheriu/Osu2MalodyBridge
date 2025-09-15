package net.omastore.malodystore.util.osuApiV2

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import okhttp3.FormBody
import okhttp3.OkHttpClient
import okhttp3.Request
import org.slf4j.LoggerFactory
import java.util.concurrent.atomic.AtomicReference
import kotlin.time.Duration.Companion.minutes

@Serializable
data class OsuClientCredentialsGrantResponse(
    @SerialName("access_token")
    val accessToken: String,
    @SerialName("token_type")
    val tokenType: String,
    @SerialName("expires_in")
    val expiresIn: Int,
)

data class AccessToken(
    val value: String,
    val issuedAt: Long,
    val expiresIn: Int,
) {
    fun isExpired(): Boolean {
        val now = System.currentTimeMillis()
        val expirationTime = issuedAt + expiresIn * 1000L
        return now > (expirationTime - 5.minutes.inWholeMilliseconds) // 提前5分钟刷新
    }
}

class Authentication(
    private val clientId: String,
    private val clientSecret: String,
    private val client: OkHttpClient = OkHttpClient(),
    private val json: Json = Json { ignoreUnknownKeys = true },
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO + SupervisorJob()),
) {
    companion object {
        @Suppress("ktlint:standard:property-naming")
        @Volatile
        private var INSTANCE: Authentication? = null

        fun getInstance(
            clientId: String,
            clientSecret: String,
            client: OkHttpClient = OkHttpClient(),
            json: Json = Json { ignoreUnknownKeys = true },
            scope: CoroutineScope = CoroutineScope(Dispatchers.IO + SupervisorJob()),
        ): Authentication =
            INSTANCE ?: synchronized(this) {
                INSTANCE ?: Authentication(clientId, clientSecret, client, json, scope).also {
                    INSTANCE = it
                }
            }
    }

    private val logger = LoggerFactory.getLogger(Authentication::class.java)

    @Suppress("ktlint:standard:backing-property-naming")
    private var _token: AtomicReference<AccessToken?> = AtomicReference(null)
    private val mutex = Mutex()

    suspend fun getAccessToken(): String? {
        var token = _token.get()
        if (token == null || token.isExpired()) {
            mutex.withLock {
                token = _token.get()
                if (token == null || token.isExpired()) {
                    val newToken = scope.async { fetchNewToken() }.await()
                    if (newToken != null) {
                        _token.set(newToken)
                        token = newToken
                    }
                }
            }
        }
        return token?.value
    }

    private suspend fun fetchNewToken(): AccessToken? =
        withContext(Dispatchers.IO) {
            val formBody =
                FormBody
                    .Builder()
                    .add("client_id", clientId)
                    .add("client_secret", clientSecret)
                    .add("grant_type", "client_credentials")
                    .add("scope", "public")
                    .build()
            val request =
                Request
                    .Builder()
                    .url("https://osu.ppy.sh/oauth/token")
                    .post(formBody)
                    .header("Accept", "application/json")
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .build()
            try {
                val response = client.newCall(request).execute()
                if (response.isSuccessful) {
                    val body = response.body.string()
                    val decodedResponse = json.decodeFromString<OsuClientCredentialsGrantResponse>(body)
                    val newToken =
                        AccessToken(
                            value = decodedResponse.accessToken,
                            issuedAt = System.currentTimeMillis(),
                            expiresIn = decodedResponse.expiresIn,
                        )
                    logger.info("OAuth2 token successfully refreshed.")
                    return@withContext newToken
                } else {
                    logger.error("Failed to fetch access token: ${response.code}")
                    null
                }
            } catch (e: Exception) {
                logger.error("Failed to authenticate with Osu! API.", e)
                null
            }
        }
}
