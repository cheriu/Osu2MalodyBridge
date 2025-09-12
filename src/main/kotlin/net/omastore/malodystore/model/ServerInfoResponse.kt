package net.omastore.malodystore.model

data class ServerInfoResponse(
    val code: Int = 0,
    val api: Int,
    val min: Int,
    val welcome: String,
)
