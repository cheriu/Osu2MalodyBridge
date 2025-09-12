package net.omastore.malodystore.config

import org.springframework.boot.context.properties.ConfigurationProperties
import org.springframework.stereotype.Component

@Component
@ConfigurationProperties(prefix = "malody.server")
data class MalodyServerProperties(
    var api: Int = 0,
    var min: Int = 0,
    var welcome: String = "",
)

@Component
@ConfigurationProperties(prefix = "malody.osu")
data class OsuAuthSecret(
    var clientId: String = "",
    var clientSecret: String = "",
)
